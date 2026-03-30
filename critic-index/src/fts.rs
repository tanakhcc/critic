//! Everything to do with searching for text in the base corpus:
//!
//! 1. Loading the base corpus into the FTS store
//! 2. Searching in the FTS store
//! 3. Searching in the full text (DB) via position finding in FTS
//! 4. Given a block of text, find the position in the base corpus

use std::sync::Arc;

use itertools::Itertools;

use critic_config::Config;
use critic_db::{get_unindexed_chunks_with_equality_alphabet, set_chunks_indexed};
use critic_format::{streamed::Block, surface_form::SurfaceBaseText};
use critic_shared::{InShutdown, urls::FTS_INDEX_BASE_LOCATION};
use tantivy::{
    Index, IndexReader, IndexWriter, Searcher, TantivyDocument, Term,
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::{BooleanQuery, Occur, PhraseQuery, QueryClone},
    schema::{FAST, Field, STORED, Schema, TEXT, Value},
};

use crate::{IndexError, OcrRecord};

const FTS_MAX_WRONG_WORDS: u8 = 2;

#[derive(Debug)]
pub enum FtsError {
    DB(critic_db::DBError),
    IO(std::io::Error),
    OpenIndex(tantivy::error::TantivyError),
    IndexWriter(tantivy::error::TantivyError),
    NewDoc(tantivy::error::TantivyError),
    IndexCommit(tantivy::error::TantivyError),
    Reader(tantivy::error::TantivyError),
    TeiXmlParse(quick_xml::de::DeError),
    TeiXmlNormalize(critic_format::denorm::NormalizationError),
    TeiXmlStream(critic_format::destream::StreamError),
    /// Failure to convert between XML and internal format
    Tei(critic_format::ConversionError),
    OpenDirectory(tantivy::directory::error::OpenDirectoryError),
    Aborted,
}
impl From<critic_format::denorm::NormalizationError> for FtsError {
    fn from(value: critic_format::denorm::NormalizationError) -> Self {
        Self::TeiXmlNormalize(value)
    }
}
impl From<critic_format::destream::StreamError> for FtsError {
    fn from(value: critic_format::destream::StreamError) -> Self {
        Self::TeiXmlStream(value)
    }
}
impl From<critic_format::ConversionError> for FtsError {
    fn from(value: critic_format::ConversionError) -> Self {
        Self::Tei(value)
    }
}
impl From<critic_db::DBError> for FtsError {
    fn from(value: critic_db::DBError) -> Self {
        Self::DB(value)
    }
}
impl From<std::io::Error> for FtsError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}
impl From<quick_xml::de::DeError> for FtsError {
    fn from(value: quick_xml::de::DeError) -> Self {
        Self::TeiXmlParse(value)
    }
}
impl From<tantivy::directory::error::OpenDirectoryError> for FtsError {
    fn from(value: tantivy::directory::error::OpenDirectoryError) -> Self {
        Self::OpenDirectory(value)
    }
}
impl core::fmt::Display for FtsError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            FtsError::IO(e) => {
                write!(f, "Error interacting with the file system: {e}")
            }
            FtsError::DB(e) => {
                write!(f, "Error interacting with the database: {e}")
            }
            FtsError::OpenIndex(e) => {
                write!(f, "Unable to open the index directory with the schema: {e}")
            }
            FtsError::IndexWriter(e) => {
                write!(f, "Unable to lock the index directory for writing: {e}")
            }
            FtsError::NewDoc(e) => {
                write!(f, "Unable to insert new document to the index: {e}")
            }
            FtsError::IndexCommit(e) => {
                write!(f, "Unable to commit a change in the index: {e}")
            }
            FtsError::Reader(e) => {
                write!(f, "Unable to get a reader from the index: {e}")
            }
            FtsError::TeiXmlParse(e) => {
                write!(f, "Unable to parse content as TEI XML: {e}")
            }
            FtsError::OpenDirectory(e) => {
                write!(f, "Unable to open mmap directory: {e}")
            }
            FtsError::Aborted => {
                write!(f, "Creation of Index was aborted.")
            }
            FtsError::TeiXmlNormalize(e) => {
                write!(f, "Failed to normalize xml: {e}")
            }
            FtsError::TeiXmlStream(e) => {
                write!(f, "Failed to stream xml: {e}")
            }
            FtsError::Tei(e) => {
                write!(f, "Failed to convert xml: {e}")
            }
        }
    }
}
impl core::error::Error for FtsError {}

fn tantivy_schema() -> Schema {
    let mut schema = Schema::builder();
    // the surface form of the text in this block
    schema.add_text_field("surface_form", TEXT | STORED);
    // the id this block has in the db
    schema.add_i64_field("id", STORED);
    schema.build()
}

/// Take the next `number_of_chunks` chunks from the DB and insert them to the index, commiting
/// afterwards.
///
/// Returns the number of chunks successfully indexed.
async fn index_next_chunks(
    config: &Config,
    number_of_chunks: i64,
    index_writer: &mut IndexWriter,
    schema: &Schema,
) -> Result<usize, FtsError> {
    let next_chunks =
        get_unindexed_chunks_with_equality_alphabet(&config.db, number_of_chunks).await?;
    let surface_form = schema.get_field("surface_form").expect("static field name");
    let id = schema.get_field("id").expect("static field name");
    for (chunk, equality_alphabet) in &next_chunks {
        let (streamed, _first_page_name) =
            critic_format::page_from_xml(chunk.content.as_bytes(), &chunk.language)?;
        let surface_base_text = SurfaceBaseText::from_blocks_with_equality_alphabet(
            &streamed,
            equality_alphabet.as_deref(),
        );
        index_writer
            .add_document(doc!(
                surface_form => surface_base_text.raw_text(),
                id => chunk.id,
            ))
            .map_err(FtsError::NewDoc)?;
    }
    let prepared = index_writer
        .prepare_commit()
        .map_err(FtsError::IndexCommit)?;
    // update the indexed status in the db
    match set_chunks_indexed(
        &config.db,
        &next_chunks.iter().map(|c| c.0.id).collect::<Vec<_>>(),
    )
    .await
    {
        Ok(()) => prepared
            .commit()
            .map(|_| next_chunks.len())
            .map_err(FtsError::IndexCommit),
        Err(e) => {
            prepared.abort().map_err(FtsError::IndexCommit)?;
            Err(e.into())
        }
    }
}

async fn create_fts_store(
    config: Arc<Config>,
    mut shutdown_rx: tokio::sync::watch::Receiver<InShutdown>,
) -> Result<IndexReader, FtsError> {
    let schema = tantivy_schema();
    let directory = MmapDirectory::open(format!(
        "{}{FTS_INDEX_BASE_LOCATION}",
        &config.data_directory
    ))?;
    let index = Index::open_or_create(directory, schema.clone()).map_err(FtsError::OpenIndex)?;
    let mut writer: IndexWriter = index.writer(25_000_000).map_err(FtsError::IndexWriter)?;
    let number_of_chunks = 32_u8;
    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                tracing::debug!("Shutting down FTS ingester now.");
                return Err(FtsError::Aborted);
            }
            next_result = index_next_chunks(&config, number_of_chunks.into(), &mut writer, &schema) => {
                match next_result {
                    Ok(indexed_chunks) => {
                        tracing::trace!("Indexed {indexed_chunks} new chunks of the base corpus.");
                        if indexed_chunks >= number_of_chunks.into() {
                            continue;
                        } else {
                            tracing::debug!("Done indexing the present base corpus. Now creating the FTS reader.");
                            return index.reader().map_err(FtsError::Reader);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Indexing new base corpus chunks failed: {e}. Waiting 1000ms.");
                        tokio::select! {
                            _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000)) => {}
                            _ = shutdown_rx.changed() => {
                                tracing::debug!("Shutting down FTS ingester now.");
                                return Err(FtsError::Aborted);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Read the base corpus from the DB and create the fts store at the right directory location.
///
/// After ingesting the base corpus, return the reader via the provided channel to use for searches later on.
pub async fn create_fts_store_with_notification(
    config: Arc<Config>,
    channel: tokio::sync::oneshot::Sender<IndexReader>,
    shutdown_tx: tokio::sync::watch::Sender<InShutdown>,
    shutdown_rx: tokio::sync::watch::Receiver<InShutdown>,
) {
    let reader = match create_fts_store(config, shutdown_rx).await {
        Ok(x) => x,
        Err(FtsError::Aborted) => {
            return;
        }
        Err(e) => {
            tracing::error!("Unable to create the FTS reader: {e}. Aborting.");
            shutdown_tx.send_replace(InShutdown::Yes);
            return;
        }
    };
    match channel.send(reader) {
        Ok(()) => {}
        Err(_) => {
            tracing::error!("Failed to send the FTS reader to the searcher task. Aborting.");
            shutdown_tx.send_replace(InShutdown::Yes);
        }
    }
}

/// levenshtein distance taken from https://github.com/rapidfuzz/strsim-rs v0.11.1
///
/// Their License is MIT.
///
/// This function acts on characters, not bytes.
fn levenshtein(a: &str, b: &str) -> usize {
    let mut cache: Vec<usize> = (1..b.len() + 1).collect();
    let mut result = b.len();

    for (i, a_elem) in a.chars().enumerate() {
        result = i + 1;
        let mut distance_b = i;

        for (j, b_elem) in b.chars().enumerate() {
            let cost = usize::from(a_elem != b_elem);
            let distance_a = distance_b + cost;
            distance_b = cache[j];
            result = std::cmp::min(result + 1, std::cmp::min(distance_a, distance_b + 1));
            cache[j] = result;
        }
    }

    result
}

/// The beginning of `haystack` is similar to `prefix`
fn prefix_is_close(prefix: &str, haystack: &str) -> bool {
    let needle_wordcount = prefix.split_whitespace().count();
    let max_search_to = core::cmp::min(haystack.len(), prefix.len() + needle_wordcount);

    if prefix.len() > max_search_to {
        return false;
    }

    let possible_endpoint = haystack.ceil_char_boundary(prefix.len());
    needle_wordcount >= levenshtein(&prefix, &haystack[0..possible_endpoint])
}

/// Find the byteindex in haystack at which needle can be found.
///
/// The level of allowed fuzzyness is controlled by prefix_is_close.
/// The range contains char indices, NOT byte indices.
///
/// The `needle` MAY NOT contain multiple whitespace characters next to each other. We will panic
/// when this is not the case.
fn fuzzy_find_needle_in_haystack(needle: &str, haystack: &str) -> Option<core::ops::Range<usize>> {
    if needle.is_empty() {
        return None;
    }

    // index of whitespace in needle and length of that whitespace
    // also contains 0,0 because that starts the first word
    let whitespace_and_size = core::iter::once((0, 0))
        .chain(needle.char_indices().filter_map(|(idx, c)| {
            (c.is_whitespace() && idx + c.len_utf8() < needle.len()).then_some((idx, c.len_utf8()))
        }))
        .collect::<Vec<_>>();

    for (word_idx, (whitespace_idx, whitespace_len)) in whitespace_and_size.iter().enumerate() {
        let index_after_word = whitespace_and_size
            .get(word_idx + 1)
            .map(|(idx, _size)| *idx)
            .unwrap_or(needle.len());
        let word = core::str::from_utf8(
            &needle.as_bytes()[*whitespace_idx + whitespace_len..index_after_word],
        )
        .expect("We have calculated the char indices at word boundaries.");

        let mut next_search_start = 0;
        while let Some(current_search_start) = haystack[next_search_start..]
            .find(word)
            .map(|idx| idx + next_search_start)
        {
            let haystack_potential_match_start = haystack.floor_char_boundary(
                current_search_start.saturating_sub(*whitespace_idx + whitespace_len),
            );
            let haystack_potential_match_end = haystack.ceil_char_boundary(core::cmp::min(
                haystack.len(),
                haystack_potential_match_start + needle.len(),
            ));
            if prefix_is_close(
                needle,
                &haystack[haystack_potential_match_start..haystack_potential_match_end],
            ) {
                return Some(haystack_potential_match_start..haystack_potential_match_end);
            } else {
                next_search_start = haystack.floor_char_boundary(core::cmp::min(
                    current_search_start + word.len() + 1,
                    haystack.len(),
                ));
            }
        }
    }
    None
}

/// The alignment information for a match from an [`OcrRecord`] in the FTS index
#[derive(Debug)]
struct FtsLineMatch {
    /// the line id that was matched
    line_index: usize,
    /// The chunk of the base corpus the match was found in
    fts_chunk: tantivy::TantivyDocument,
    /// The ID of the doc in the database (this is doc -> id; we return it here to prevent
    /// unnecessary FS operations to read this ID later on)
    fts_chunk_id: i64,
    /// The content in the FTS chunk that matched the [`OcrRecord`]
    ///
    /// This index is a byte position
    in_chunk_position: core::ops::Range<usize>,
}

/// Find `line` in the FTS index. Only full inclusions in a single chunk are considered.
///
/// When the line overlaps (part is in one chunk, another part is in another chunk), this method
/// will not find it.
async fn line_match_in_fts(
    searcher: &Searcher,
    line: &OcrRecord,
    line_index: usize,
    body: Field,
    id: Field,
) -> Result<Option<FtsLineMatch>, IndexError> {
    let subqueries = line
        .prediction
        .split_whitespace()
        .tuple_windows()
        .map(|(w1, w2)| {
            let mut terms = Vec::with_capacity(2);
            terms.push(Term::from_field_text(body, &w1.to_lowercase()));
            terms.push(Term::from_field_text(body, &w2.to_lowercase()));
            let phrase_query = PhraseQuery::new(terms);
            (Occur::Should, phrase_query.box_clone())
        })
        .collect::<Vec<_>>();
    let minimum_should = core::cmp::max(
        subqueries
            .len()
            .saturating_sub(2 * FTS_MAX_WRONG_WORDS as usize),
        1,
    );
    let mut query = BooleanQuery::new(subqueries);
    query.set_minimum_number_should_match(minimum_should);

    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(1))
        .map_err(IndexError::FtsSearch)?;
    let Some((_score, doc_address)) = top_docs.get(0) else {
        return Ok(None);
    };
    let doc = searcher
        .doc::<TantivyDocument>(*doc_address)
        .map_err(IndexError::OpenDocument)?;
    // now get the location of the found string in the documents body
    let content = doc
        .get_first(body)
        .expect("schema is static")
        .as_str()
        .expect("body is a string");
    let Some(position) = fuzzy_find_needle_in_haystack(&line.prediction, content) else {
        return Ok(None);
    };
    let id = doc
        .get_first(id)
        .expect("schema is static")
        .as_i64()
        .expect("id is an i64");

    Ok(Some(FtsLineMatch {
        line_index,
        fts_chunk: doc,
        fts_chunk_id: id,
        in_chunk_position: position,
    }))
}

/// Given the proposed text on a page, find the associated content in the base corpus.
///
/// Each [`OcrRecord`] represents one line, and the output contains one entry with inline
/// [`Block`]s for that line.
pub async fn basetext_from_proposal(
    config: &Config,
    searcher: Searcher,
    proposal: &[OcrRecord],
) -> Result<Vec<Vec<Block>>, IndexError> {
    let schema = searcher.schema();
    let body = schema
        .get_field("surface_form")
        .map_err(IndexError::WrongSchema)?;
    let id = schema.get_field("id").map_err(IndexError::WrongSchema)?;
    // get best match for each line
    let mut line_matches = Vec::new();

    for (idx, line) in proposal.iter().enumerate() {
        if let Some(line_match) = line_match_in_fts(&searcher, line, idx, body, id).await? {
            tracing::trace!("Actually found a line match.");
            line_matches.push(line_match);
        };
    }
    let chunks = critic_db::get_base_corpus_chunks_by_id(
        &config.db,
        &line_matches
            .iter()
            .map(|lm| lm.fts_chunk_id)
            .collect::<Vec<i64>>(),
    )
    .await
    .map_err(IndexError::DB)?;

    let mut res = vec![Vec::default(); proposal.len()];
    // fill res with the known line matches
    for line_match in line_matches {
        let our_chunk = chunks
            .iter()
            .find(|chunk| chunk.id == line_match.fts_chunk_id)
            .expect("We successfully got all chunks from the DB earlier.");
        // get the part of the base corpus chunk that actually belongs to this line
        let surface_form = SurfaceBaseText::from_blocks_with_equality_alphabet(
            &our_chunk.content,
            our_chunk.equality_alphabet.as_deref(),
        );
        let Some(starting_index) = surface_form.indexmap().iter().find(|&surface_index| {
            surface_form
                .raw_text()
                .char_indices()
                .nth(surface_index.position_in_raw())
                .is_some_and(|(idx, _char)| idx == line_match.in_chunk_position.start)
        }) else {
            continue;
        };
        let mut tei_match: Vec<_> = our_chunk
            .content
            .iter()
            .skip(starting_index.block_position())
            .map(|b| b.clone())
            .collect();
        tei_match.first_mut().map(|b| match b {
            Block::Text(paragraph) => {
                paragraph.content = paragraph
                    .content
                    .split_whitespace()
                    .skip(starting_index.position_in_block())
                    .join(" ");
            }
            Block::Uncertain(uncertain) => {
                uncertain.content = uncertain
                    .content
                    .split_whitespace()
                    .skip(starting_index.position_in_block())
                    .join(" ");
            }
            Block::Correction(correction) => {
                correction.versions.last_mut().map(|v| {
                    v.content = v
                        .content
                        .split_whitespace()
                        .skip(starting_index.position_in_block())
                        .join(" ")
                });
            }
            Block::Abbreviation(abbreviation) => {
                abbreviation.surface = abbreviation
                    .surface
                    .split_whitespace()
                    .skip(starting_index.position_in_block())
                    .join(" ");
            }
            Block::Anchor(_) | Block::Break(_) | Block::Lacuna(_) | Block::Space(_) => {}
        });
        res[line_match.line_index] = tei_match;
    }
    // now infill between known line matches
    //      when the two adjacent elements are the same chunk, return everything in between
    //      when the adjacent elements are next to each other in at least one versification
    //        scheme, return the end of the first and the start of the next base corpus chunk
    return Ok(res);

    // fill gaps in the middle
    // extrapolate for leading/trailing lines with no match
}

#[cfg(test)]
mod test {
    use super::fuzzy_find_needle_in_haystack;

    #[test]
    fn fuzzy_find_exact() {
        let needle = "abc";
        let haystack = "01234 abc 01324";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, Some(6..9));
    }

    #[test]
    fn fuzzy_find_double() {
        let needle = "abc";
        let haystack = "01234 abc 01324 abc";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, Some(6..9));
    }

    #[test]
    fn fuzzy_find_no_match() {
        let needle = "someone without sin";
        let haystack = "The world";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, None);
    }

    /// Since we need an exact match for at least one word, a single misspelled word will never be
    /// found.
    #[test]
    fn fuzzy_find_single_word_fuzzy() {
        let needle = "word";
        let haystack = "The world is a cruel place";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, None);
    }

    #[test]
    fn fuzzy_find_two_word_fuzzy() {
        let needle = "The word";
        let haystack = "The world is a cruel place";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, Some(0..8));
    }

    #[test]
    fn fuzzy_find_very_wrong() {
        let needle = "The warudo";
        let haystack = "The world is a cruel place";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, None);
    }

    #[test]
    fn fuzzy_find_long_scalar() {
        let needle = "The world";
        let haystack = "T\u{10ffff}e world is a cruel place";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, None);
    }

    #[test]
    fn fuzzy_find_long_scalar_in_needle() {
        let needle = "מִ";
        let haystack = "A אפנמ word מִאדשג aword שכהנ";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, Some(16..20));
    }

    #[test]
    fn fuzzy_find_multiple_anchors() {
        let needle = "I am a needle.";
        let haystack = "I am the haystack. I am a nedle. I am another haystack.";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, Some(19..33));
    }

    #[test]
    fn fuzzy_find_complicated_anchor() {
        let needle = "Consubstantiation is quite a complicated word.";
        let haystack = "Consubstantiation is one thing, but Transubstantiation another. Anyways, this haystack does not contain the complicated Needle.";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, None);
    }

    #[test]
    fn fuzzy_find_match_at_haystack_end() {
        let needle = "Needle";
        let haystack = "Haystack Another Haystack. Funny Easteregg. Needle.";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, Some(44..50));

        let needle = "Needle";
        let haystack = "Haystack Another Haystack. Needle";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, Some(27..33));
    }

    #[test]
    fn fuzzy_find_fuzzy_at_haystack_end() {
        let needle = "The Nedle";
        let haystack = "Haystack Another Haystack. The Needle";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, Some(27..36));
    }

    #[test]
    fn fuzzy_find_long_haystack() {
        let haystack = "Als nun die Tage Davids herannahten, dass er sterben sollte, befahl er seinem Sohn Salomo und sagte: Ich gehe nun den Weg aller Welt. So sei stark und erweise dich als Mann! Bewahre, was der HERR, dein Gott, zu bewahren geboten hat, dass du auf seinen Wegen gehst, indem du seine Ordnungen, seine Gebote und seine Rechtsbestimmungen und seine Zeugnisse bewahrst, wie es im Gesetz des Mose geschrieben ist, damit du Erfolg hast in allem, was du tust, und überall, wohin du dich wendest; damit der HERR sein Wort aufrechterhält, das er über mich geredet hat, als er sprach: Wenn deine Söhne auf ihren Weg achthaben, sodass sie in Treue vor mir leben mit ihrem ganzen Herzen und mit ihrer ganzen Seele, dann soll es dir nicht an einem Mann fehlen auf dem Thron Israels. Auch hast du ja selbst erkannt, was mir Joab, der Sohn der Zeruja, angetan hat, was er den beiden Heerobersten Israels, Abner, dem Sohn Ners, und Amasa, dem Sohn Jeters, angetan hat, dass er sie ermordete und so mit Kriegsblut den Frieden belastete. So hat er Kriegsblut an seinen Gürtel gebracht, der um seine Hüften war, und an seine Schuhe, die an seinen Füßen waren. So handle nun nach deiner Weisheit und lass sein graues Haar nicht in Frieden in den Scheol hinabfahren! Aber an den Söhnen des Gileaditers Barsillai sollst du Gnade erweisen, und sie sollen unter denen sein, die an deinem Tisch essen; denn ebenso sind sie mir entgegengekommen, als ich vor deinem Bruder Absalom floh. Und siehe, bei dir ist Schimi, der Sohn Geras, der Benjaminiter aus Bahurim; das ist der, der mich mit einem schlimmen Fluch verflucht hat am Tag, als ich nach Mahanajim ging. Aber er kam mir dann entgegen, an den Jordan herab, und ich schwor ihm bei dem HERRN und sprach: Wenn ich dich mit dem Schwert töte! Jetzt aber lass ihn nicht ungestraft, denn du bist ein weiser Mann und wirst erkennen, was du ihm tun sollst. Lass sein graues Haar mit Blut befleckt in den Scheol hinabfahren! Und David legte sich zu seinen Vätern und wurde in der Stadt Davids begraben. Und die Tage, die David über Israel König war, betrugen vierzig Jahre. In Hebron war er sieben Jahre König, und in Jerusalem war er 33 Jahre König. Und Salomo setzte sich auf den Thron seines Vaters David, und seine Königsherrschaft war fest gegründet.";
        let needle = "Und seine Königsherrschaft waret fest gegründet.";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, Some(2240..2290));

        let needle = "Und seine war fest gegründet.";
        let found = fuzzy_find_needle_in_haystack(needle, haystack);
        assert_eq!(found, None);
    }
}
