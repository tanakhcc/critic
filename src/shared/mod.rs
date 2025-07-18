//! Types and functions shared by App and Server

use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;

#[cfg(feature = "ssr")]
pub mod auth;

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
