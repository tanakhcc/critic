//! Critic README TODO

use oauth2::TokenResponse;
use serde::Deserialize;

#[cfg(feature = "ssr")]
pub mod auth;

#[cfg(feature = "ssr")]
mod config;

#[cfg(feature = "ssr")]
mod db;

// some basic types used across the app
/// The JSON object returned from gitlabs get-user endpoint
#[derive(Debug, Deserialize)]
struct UserInfo {
    /// ID of the user in gitlab - we use the same ID in the internal DB here
    id: i32,
    /// username of the user in gitlab - we use the same here
    username: String,
}
impl From<AuthenticatedUser> for UserInfo {
    fn from(value: AuthenticatedUser) -> Self {
        Self {
            id: value.id,
            username: value.username,
        }
    }
}

/// The full User with oauth2 credentials
#[derive(Deserialize, Clone, sqlx::prelude::FromRow)]
pub struct AuthenticatedUser {
    id: i32,
    username: String,
    access_token: String,
    refresh_token: String,
    expires_at: time::OffsetDateTime,
}
impl std::fmt::Debug for AuthenticatedUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthenticatedUser")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("access_token", &"[redacted]")
            .field("refresh_token", &"[redacted]")
            .field("expires_at", &self.expires_at)
            .finish()
    }
}



#[derive(Debug)]
enum NormalizeTokenResponseError {
    NoRefresh,
    NoExpiresIn,
}
impl core::fmt::Display for NormalizeTokenResponseError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::NoRefresh => {write!(f, "No refresh token was given")}
            Self::NoExpiresIn => {write!(f, "No expires_in time was given")}
        }
    }
}
impl std::error::Error for NormalizeTokenResponseError {}
#[derive(Debug)]
struct NormalizedTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_at: time::OffsetDateTime,
}
impl TryFrom<oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>> for NormalizedTokenResponse {
    type Error = NormalizeTokenResponseError;

    fn try_from(value: oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>) -> Result<Self, Self::Error> {
        let expires_at = time::OffsetDateTime::now_utc() + value.expires_in().ok_or(NormalizeTokenResponseError::NoExpiresIn)?;
        Ok(Self {
            access_token: value.access_token().clone().into_secret(),
            refresh_token: value.refresh_token().ok_or(NormalizeTokenResponseError::NoRefresh)?.clone().into_secret(),
            expires_at,
        })
    }
}


#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use crate::auth::GitlabOauthBackend;
    use axum::Router;
    use axum_login::{login_required, tower_sessions::{Expiry, MemoryStore, SessionManagerLayer}, AuthManagerLayerBuilder};
    use critic::app::*;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use time::Duration;
    use tracing::{debug, info};
    use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

    let config = match config::Config::try_create().await {
        Ok(x) => x,
        Err(e) => {
            panic!("Error reading config: {e}.");
        }
    };
    let config_arc = Arc::new(config);

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    match sqlx::migrate!().run(&config_arc.db).await {
        Ok(_) => {},
        Err(e) => {
            panic!("Error migrating database: {e}");
        }
    }

    let my_crate_filter = EnvFilter::new("critic");
    let subscriber = tracing_subscriber::registry().with(my_crate_filter).with(
        tracing_subscriber::fmt::layer()
            .compact()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_line_number(true)
            .with_filter(config_arc.log_level),
    );
    tracing::subscriber::set_global_default(subscriber).expect("static tracing config");
    debug!("Tracing enabled.");

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    // we want to build our Router:
    // protected (including /api/protected/*server_fn)
    // login layer
    // leptos_routes_with_exclusions (exclude protected and login layer) - this generates all other
    // leptos routes
    let app_core = Router::new()
        .leptos_routes(&config_arc.leptos_options, routes, {
            let leptos_options = config_arc.leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(config_arc.leptos_options.clone())
        ;

    // create the auth layer on top of our application core
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(axum_login::tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));
    let backend = GitlabOauthBackend::new(config_arc.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = app_core
        .route_layer(login_required!(GitlabOauthBackend, login_url = "/login"))
        .merge(crate::auth::backend::auth_router())
        .layer(auth_layer);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    info!("listening on http://{}", &config_arc.leptos_options.site_addr);
    let listener = tokio::net::TcpListener::bind(&config_arc.leptos_options.site_addr)
        .await
        .unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
