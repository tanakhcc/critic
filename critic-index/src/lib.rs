//! Lower level implementations for segmentation, ocr and indexing in critic.

use std::sync::Arc;

use critic_config::Config;
use critic_shared::InShutdown;
use tantivy::IndexReader;

pub mod fts;
pub mod segment;

#[derive(Debug)]
pub enum IndexError {
    /// something went wrong while calling to kraken
    Kraken(pyo3::PyErr),
    /// something went wrong while casting between python types
    Cast(String),
    /// The return type from kraken.blla.segment should contains `.regions['text']`.
    /// When 'text' is not found, this error is raised.
    NoTextInRegion,
}
impl From<pyo3::PyErr> for IndexError {
    fn from(value: pyo3::PyErr) -> Self {
        IndexError::Kraken(value)
    }
}
impl core::fmt::Display for IndexError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            IndexError::Kraken(e) => {
                write!(f, "Error in python: {e}")
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
        }
    }
}
impl core::error::Error for IndexError {}

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
    tracing::info!("I would now start indexing pages");
}
