//! Everything related to storing raw transcriptions on disk on the server.

use std::io::Write;
use std::path::PathBuf;

use critic_format::{
    denorm::NormalizationError, destream::StreamError, streamed::Manuscript, ConversionError,
};
use critic_shared::urls::TRANSCRIPTION_BASE_LOCATION;

/// Anything that can go wrong while reading or writing Transcriptions to disk
#[derive(Debug)]
pub enum TranscriptionStoreError {
    // Path - Problem
    Open(String, std::io::Error),
    // Path - Problem
    Write(String, std::io::Error),
    CreateDir(String, std::io::Error),
    // Deserialization (i.e. Syntax-Errors)
    // Path - Problem
    Deser(String, quick_xml::DeError),
    // Serialization (i.e. Syntax-Errors)
    Ser(quick_xml::SeError),
    Stream(StreamError),
    DeStream(StreamError),
    Norm(NormalizationError),
    DeNorm(NormalizationError),
}
impl core::fmt::Display for TranscriptionStoreError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Open(p, e) => {
                write!(f, "Failed to open {p}: {e}")
            }
            Self::Write(p, e) => {
                write!(f, "Failed to write to {p}: {e}")
            }
            Self::CreateDir(p, e) => {
                write!(f, "Failed to create directory {p}: {e}")
            }
            Self::Deser(p, e) => {
                write!(f, "Failed to deserialize from {p}: {e}")
            }
            Self::Ser(e) => {
                write!(f, "Failed to serialize: {e}")
            }
            Self::Stream(e) => {
                write!(f, "Failed to stream transcription: {e}")
            }
            Self::DeStream(e) => {
                write!(f, "Failed to destream transcription: {e}")
            }
            Self::Norm(e) => {
                write!(f, "Failed to normalize transcription: {e}")
            }
            Self::DeNorm(e) => {
                write!(f, "Failed to denormalize transcription: {e}")
            }
        }
    }
}
impl core::error::Error for TranscriptionStoreError {}

pub fn read_transcription_from_disk(
    data_directory: &str,
    msname: &str,
    pagename: &str,
    username: &str,
) -> Result<Manuscript, TranscriptionStoreError> {
    let mut path = PathBuf::new();
    path.push(data_directory);
    path.push(&TRANSCRIPTION_BASE_LOCATION[1..]);
    path.push(msname);
    path.push(pagename);
    path.push(username);
    path.set_extension("xml");
    let file = match std::fs::File::open(&path) {
        Ok(x) => x,
        Err(e) => {
            return Err(TranscriptionStoreError::Open(
                path.to_string_lossy().to_string(),
                e,
            ));
        }
    };
    let buf_reader = std::io::BufReader::new(file);
    critic_format::from_xml(buf_reader).map_err(|e| match e {
        ConversionError::DeSer(de_err) => {
            TranscriptionStoreError::Deser(path.to_string_lossy().to_string(), de_err)
        }
        ConversionError::Norm(norm_err) => TranscriptionStoreError::Norm(norm_err),
        ConversionError::Stream(stream_err) => TranscriptionStoreError::Stream(stream_err),
        _ => {
            unreachable!()
        }
    })
}

/// We have already checked that we really want to save this transcription data.
/// Write it to disk.
pub fn write_transcription_to_disk(
    data: Manuscript,
    data_directory: &str,
    username: &str,
) -> Result<(), TranscriptionStoreError> {
    let mut path = PathBuf::new();
    path.push(data_directory);
    path.push(&TRANSCRIPTION_BASE_LOCATION[1..]);
    path.push(&data.meta.title);
    path.push(&data.meta.page_nr);
    std::fs::create_dir_all(&path)
        .map_err(|e| TranscriptionStoreError::CreateDir(path.to_string_lossy().to_string(), e))?;
    path.push(username);
    path.set_extension("xml");
    let mut file = match std::fs::OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .open(&path)
    {
        Ok(x) => x,
        Err(e) => {
            return Err(TranscriptionStoreError::Open(
                path.to_string_lossy().to_string(),
                e,
            ));
        }
    };

    let sr = critic_format::to_xml(data).map_err(|e| match e {
        ConversionError::Ser(e) => TranscriptionStoreError::Ser(e),
        ConversionError::DeNorm(e) => TranscriptionStoreError::DeNorm(e),
        ConversionError::DeStream(e) => TranscriptionStoreError::DeStream(e),
        _ => {
            unreachable!()
        }
    })?;

    file.write(sr.as_bytes())
        .map(|_| ())
        .map_err(|e| TranscriptionStoreError::Write(path.to_string_lossy().to_string(), e))
}
