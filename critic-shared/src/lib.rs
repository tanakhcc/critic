//! Types and functions shared by App and Server

pub mod urls;

use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;

/// Response from the backend when a file transfer succeeded
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct FileTransferOkResponse {
    pub new_pages: i32,
}
impl core::fmt::Display for FileTransferOkResponse {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.new_pages)
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
