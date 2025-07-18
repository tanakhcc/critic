//! Getting the versification scheme from the DB

use leptos::prelude::*;
use leptos::{prelude::ServerFnError, server};
use serde::{Deserialize, Serialize};

/// The names of a versification scheme
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct VersificationScheme {
    id: i64,
    /// The full name, e.g. "Present"
    pub full_name: String,
    /// The shorthand, e.g. "P"
    pub shorthand: String,
}

#[cfg(feature = "ssr")]
/// used internally for talking to postgres
#[derive(sqlx::FromRow)]
struct __VersificationScheme {
    id: i64,
    /// The full name, e.g. "Present"
    full_name: String,
    /// The shorthand, e.g. "P"
    shorthand: String,
}
#[cfg(feature = "ssr")]
impl From<__VersificationScheme> for VersificationScheme {
    fn from(value: __VersificationScheme) -> Self {
        Self {
            id: value.id,
            full_name: value.full_name,
            shorthand: value.shorthand,
        }
    }
}

#[server]
pub async fn get_versification_schemes() -> Result<Vec<VersificationScheme>, ServerFnError> {
    use sqlx::query_as;

    let db_pool: sqlx::Pool<sqlx::Postgres> =
        use_context().ok_or(ServerFnError::new("Unable to get db_pool from context."))?;

    Ok(
        query_as!(__VersificationScheme, "SELECT * FROM versification_scheme;")
            .fetch_all(&db_pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Unable to get versification schemes: {e}")))?
            .into_iter()
            .map(Into::<VersificationScheme>::into)
            .collect(),
    )
}
