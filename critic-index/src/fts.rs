//! Everything to do with searching for text in the base corpus:
//!
//! 1. Loading the base corpus into the FTS store
//! 2. Searching in the FTS store
//! 3. Searching in the full text (DB) via position finding in FTS
//! 4. Given a block of text, find the position in the base corpus

use std::sync::Arc;

use critic_config::Config;
use critic_db::{
    get_unindexed_chunks, get_unindexed_chunks_with_equality_alphabet, set_chunks_indexed,
};
use critic_format::{streamed::Block, surface_form::SurfaceBaseText};
use critic_shared::{InShutdown, urls::FTS_INDEX_BASE_LOCATION};
use tantivy::{
    Index, IndexReader, IndexWriter, Searcher, TantivyDocument,
    directory::MmapDirectory,
    doc,
    schema::{FAST, STORED, Schema, TEXT},
};

use crate::{IndexError, OcrRecord};

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
        let parsed: critic_format::schema::Text = quick_xml::de::from_str(&chunk.content)?;
        let normed: critic_format::normalized::Text = parsed.try_into()?;
        let streamed: Vec<critic_format::streamed::Block> = normed.try_into()?;
        let surface_base_text =
            SurfaceBaseText::from_blocks_with_equality_alphabet(&streamed, Some(equality_alphabet));
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
                        tracing::debug!("Indexed {indexed_chunks} new chunks of the base corpus.");
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

/// Given the proposed text on a page, find the associated content in the base corpus.
///
/// Each [`OcrRecord`] represents one line, and the output contains one entry with inline
/// [`Block`]s for that line.
pub async fn basetext_from_proposal(
    config: &Config,
    searcher: Searcher,
    proposal: &[OcrRecord],
) -> Result<Vec<Vec<Block>>, IndexError> {
    todo!()
}
