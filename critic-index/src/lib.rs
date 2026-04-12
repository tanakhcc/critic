//! Lower level implementations for segmentation, ocr and indexing in critic.

use std::sync::Arc;

use critic_config::Config;
use critic_db::{DBError, get_next_kraken_task};
use critic_shared::{Baseline, InShutdown, Point, Region, RegionType};
use itertools::Itertools;
use ocr::handle_ocr_task;
use pyo3::FromPyObject;
use segment::handle_baseline_task;
use tantivy::{IndexReader, TantivyError};

pub mod fts;
pub mod ocr;
pub mod segment;

#[derive(Debug, FromPyObject)]
struct KrakenBaseline {
    id: String,
    baseline: Vec<Vec<u32>>,
    boundary: Vec<Vec<u32>>,
}

/// Error when the baseline is not a list of exactly two items
impl TryFrom<KrakenBaseline> for Baseline {
    type Error = ();

    fn try_from(value: KrakenBaseline) -> Result<Self, Self::Error> {
        Ok(Baseline {
            id: 0,
            baseline: (
                {
                    let entry = value.baseline.get(0).ok_or(())?;
                    if entry.len() != 2 {
                        return Err(());
                    }
                    Point {
                        x: entry[0],
                        y: entry[1],
                    }
                },
                {
                    let entry = value.baseline.get(1).ok_or(())?;
                    if entry.len() != 2 {
                        return Err(());
                    }
                    Point {
                        x: entry[0],
                        y: entry[1],
                    }
                },
            ),
            boundary: value.boundary.try_into()?,
            content: Vec::with_capacity(1),
        })
    }
}

#[derive(Debug, FromPyObject)]
struct KrakenRegion {
    id: String,
    boundary: Vec<Vec<u32>>,
}
impl KrakenRegion {
    fn try_into_region(self, region_type: RegionType) -> Result<Region, ()> {
        Ok(Region {
            id: 0,
            region_type: region_type,
            boundary: self.boundary.try_into()?,
            baselines: Vec::default(),
        })
    }
}

/// The Result after running OCR over a single line.
#[derive(Debug)]
pub struct OcrRecord {
    /// The predicted text as a continuous string.
    ///
    /// Whitspace is normalized to single spaces.
    prediction: String,
    /// The associated baseline
    baseline: (Point, Point),
}
impl OcrRecord {
    pub fn new(
        prediction_as_str: &str,
        baseline: (Point, Point),
        equality_alphabet: Option<&str>,
    ) -> Self {
        OcrRecord {
            prediction: prediction_as_str
                .split_whitespace()
                .map(|word| {
                    word.replace(|c| equality_alphabet.is_some_and(|ea| !ea.contains(c)), "")
                })
                .join(" "),
            baseline,
        }
    }
}

#[derive(Debug)]
pub enum IndexError {
    /// something went wrong while calling to kraken
    Kraken(pyo3::PyErr),
    /// something went wrong while casting between python types
    Cast(String),
    /// The return type from kraken.blla.segment should contain `.regions['text']`.
    /// When 'text' is not found, this error is raised.
    NoTextInRegion,
    /// Something went wrong while talking to the DB
    DB(DBError),
    /// A regions boundary we got from python is not in the correct format.
    RegionFormat,
    /// A baselines boundary we got from python is not in the correct format.
    BaselineFormat,
    /// We have no OCR model available for manuscript, page
    NoOcrModel(String, String),
    /// We have no Segmentation model available for manuscript, page
    NoSegmentationModel(String, String),
    /// Unable to search in the FTS index
    FtsSearch(TantivyError),
    /// Unable to open a document after the search
    OpenDocument(TantivyError),
    /// The index has the wrong schema
    WrongSchema(TantivyError),
    /// Failed to convert from or to TEI XML
    Tei(critic_format::ConversionError),
    /// Failed to Join Kraken Job (the job paniced)
    Join(tokio::task::JoinError),
    /// The region Type was unknown
    UnknownRegionType,
    /// A SurfaceIndex indicates a character position that is higher then the number of characters
    /// in the base text.
    PositionInRawImpossible,
}
impl From<pyo3::PyErr> for IndexError {
    fn from(value: pyo3::PyErr) -> Self {
        IndexError::Kraken(value)
    }
}
impl From<DBError> for IndexError {
    fn from(value: DBError) -> Self {
        IndexError::DB(value)
    }
}
impl From<tokio::task::JoinError> for IndexError {
    fn from(value: tokio::task::JoinError) -> Self {
        IndexError::Join(value)
    }
}
impl core::fmt::Display for IndexError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            IndexError::Kraken(e) => {
                write!(f, "Error in python: {e:?}.")
            }
            IndexError::Cast(e) => {
                write!(f, "Error Casting between python types: {e}")
            }
            IndexError::NoTextInRegion => {
                write!(
                    f,
                    "Found no text entry in region dict returned from blla.segment."
                )
            }
            IndexError::DB(e) => {
                write!(f, "Problem while communicating with the DB: {e}")
            }
            IndexError::RegionFormat => {
                write!(
                    f,
                    "Got a region from kraken that is not list[tuple(int, int)]."
                )
            }
            IndexError::BaselineFormat => {
                write!(
                    f,
                    "Got a baseline from kraken that is not list[2; tuple(int, int)]."
                )
            }
            IndexError::NoOcrModel(ms, page) => {
                write!(f, "There is no OCR model present for MS {ms}, page {page}.")
            }
            IndexError::NoSegmentationModel(ms, page) => {
                write!(f, "There is no OCR model present for MS {ms}, page {page}.")
            }
            IndexError::OpenDocument(e) => {
                write!(f, "Unable to open a document from the FTS store: {e}.")
            }
            IndexError::FtsSearch(e) => {
                write!(f, "Unable to search through the FTS index: {e}.")
            }
            IndexError::WrongSchema(e) => {
                write!(f, "The FTS index has the wrong schema: {e}.")
            }
            IndexError::Tei(e) => {
                write!(f, "Failed to convert from or to XML: {e}")
            }
            IndexError::Join(e) => {
                write!(f, "Kraken Thread paniced: {e}")
            }
            IndexError::UnknownRegionType => {
                write!(f, "The region type was unknown.")
            }
            IndexError::PositionInRawImpossible => {
                write!(f, "A SurfaceIndex points to a character position in the cleaned base corpus chunk that is higher then the number of chars in that base corpus chunk.
                    This is a programmer error.")
            }
        }
    }
}
impl core::error::Error for IndexError {}

/// Continuously runs the next available kraken task in the db.
///
/// This is biased to try OCR tasks before basline segmentation tasks.
pub async fn run_kraken(
    config: Arc<Config>,
    mut shutdown_rx: tokio::sync::watch::Receiver<InShutdown>,
    index_reader: IndexReader,
) {
    loop {
        let task = match get_next_kraken_task(&config.db).await {
            Ok(Some(x)) => x,
            Ok(None) => {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        tracing::debug!("Shutting down kraken runner.");
                        return;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                        continue;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed getting the next kraken task: {e}. Waiting for 1s.");
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        tracing::debug!("Shutting down kraken runner.");
                        return;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                        continue;
                    }
                }
            }
        };
        match task {
            critic_db::KrakenTask::Ocr(task) => {
                match handle_ocr_task(&config, &task, index_reader.searcher()).await {
                    Ok(()) => {}
                    Err(e) => {
                        tracing::warn!(
                            "Failed to baseline and index the page {} - {}: {e}. Waiting for 1s.",
                            task.manuscript,
                            task.page
                        );
                        tokio::select! {
                            _ = shutdown_rx.changed() => {
                                tracing::debug!("Shutting down kraken runner.");
                                return;
                            }
                            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                                continue;
                            }
                        }
                    }
                }
            }
            critic_db::KrakenTask::Baseline(task) => {
                if let Err(e) = handle_baseline_task(&config, &task).await {
                    tracing::warn!(
                        "Failed to automatically baseline the page {} - {}: {e}. Waiting for 1s.",
                        task.manuscript,
                        task.page
                    );
                    tokio::select! {
                        _ = shutdown_rx.changed() => {
                            tracing::debug!("Shutting down kraken runner.");
                            return;
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                            continue;
                        }
                    }
                }
            }
        }
    }
}

/// Continuously indexes pages.
///
/// Pulls pages to index from the db that require OCR to be run.
/// - tries to index the page
/// - writes the best-guess basetext into the db
pub async fn run_indexing(
    config: Arc<Config>,
    fts_reader_rx: tokio::sync::oneshot::Receiver<IndexReader>,
    mut shutdown_rx: tokio::sync::watch::Receiver<InShutdown>,
    shutdown_tx: tokio::sync::watch::Sender<InShutdown>,
) {
    let index_reader = tokio::select! {
        _ = shutdown_rx.changed() => {
            tracing::debug!("Shutting down continuous indexing service now.");
            return;
        }
        res = fts_reader_rx => {
            match res {
                Ok(x) => x,
                Err(e) => {
                    tracing::error!(
                        "Failed to receive the FTS reader in the indexing thread: {e}. Aborting."
                    );
                    shutdown_tx.send_replace(InShutdown::Yes);
                    return;
                }
            }
        }
    };
    run_kraken(config, shutdown_rx, index_reader).await
}
