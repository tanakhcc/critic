//! Types and functions shared by App and Server

pub mod urls;

use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;

/// The extensions that we allow for page images
pub const ALLOWED_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg"];
/// Max body size for POST-requests in bytes
///
/// Please note changes to this value in the README under `Reverse Proxying critic`
pub const MAX_BODY_SIZE: usize = 150 * 1024 * 1024;

/// Response from the backend after file uploads
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct FileTransferResponse {
    pub err: Vec<Option<String>>,
}
impl FileTransferResponse {
    pub fn new() -> Self {
        Self { err: Vec::new() }
    }

    /// A single file was uploaded ok
    pub fn push_ok(&mut self) {
        self.err.push(None);
    }
    /// A bunch of files were uploaded ok
    pub fn push_ok_batch(&mut self, batch_size: usize) {
        self.err.extend(std::iter::repeat_n(None, batch_size));
    }
    /// There was a problem uploading the next file
    pub fn push_err(&mut self, error: String) {
        self.err.push(Some(error));
    }
    /// There was the same problem uploading a bunch of files
    pub fn push_err_batch(&mut self, error: String, batch_size: usize) {
        self.err
            .extend(std::iter::repeat_n(Some(error), batch_size));
    }
}
impl Extend<Option<String>> for FileTransferResponse {
    fn extend<T: IntoIterator<Item = Option<String>>>(&mut self, iter: T) {
        self.err.extend(iter);
    }
}

/// The names of a versification scheme
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct VersificationScheme {
    pub id: i64,
    /// The full name, e.g. "Present"
    pub full_name: String,
    /// The shorthand, e.g. "P"
    pub shorthand: String,
}

/// Metainformation on manuscripts
#[cfg_attr(feature = "ssr", derive(FromRow))]
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct ManuscriptMeta {
    pub id: i64,
    /// Title of this manuscript
    pub title: String,
    pub institution: Option<String>,
    pub collection: Option<String>,
    pub hand_desc: Option<String>,
    pub script_desc: Option<String>,
}

/// complete information for a manuscript, including its pages
#[cfg_attr(feature = "ssr", derive(FromRow))]
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Manuscript {
    pub meta: ManuscriptMeta,
    pub pages: Vec<PageMeta>,
}

/// complete information for an individual page
#[cfg_attr(feature = "ssr", derive(FromRow))]
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct PageMeta {
    pub id: i64,
    pub manuscript_id: i64,
    pub name: String,
    pub verse_start: Option<i64>,
    pub verse_end: Option<i64>,
}

/// This provides context through the entire app. When ShowHelp(true) is present, some components
/// show a help-text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowHelp(bool);
impl ShowHelp {
    pub fn new(active: bool) -> Self {
        Self(active)
    }
    pub fn toggle(&mut self) {
        self.0 ^= true
    }
    pub fn set_off(&mut self) {
        self.0 = false
    }
    pub fn get(&self) -> bool {
        self.0
    }
}
impl From<ShowHelp> for bool {
    fn from(value: ShowHelp) -> Self {
        value.0
    }
}
