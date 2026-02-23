//! Lower level implementations for segmentation, ocr and indexing in critic.

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
