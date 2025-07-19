//! Getting the versification scheme from the DB

use critic_shared::VersificationScheme;
use leptos::{prelude::ServerFnError, server};

#[server]
pub async fn get_versification_schemes() -> Result<Vec<VersificationScheme>, ServerFnError> {
    use leptos::prelude::use_context;
    let config: std::sync::Arc<critic_server::config::Config> =
        use_context().ok_or(ServerFnError::new("Unable to get config from context"))?;
    critic_server::db::get_versification_schemes(&config.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
