//! The backend endpoints for authentication and session handling for the oauth flow

/// keys used in the session store
const CSRF_STATE_KEY: &str = "oauth.csrf-state";
const PKCE_VERIFIER_KEY: &str = "oauth.pkce_verifier";
const NEXT_URL_KEY: &str = "oauth.next";

use axum::{extract::Query, response::IntoResponse, Router};
use axum_login::tower_sessions::Session;
use oauth2::{CsrfToken, PkceCodeVerifier};
use reqwest::StatusCode;
use serde::Deserialize;
use tracing::{error, warn};

use super::{AuthSession, Credentials};

pub fn auth_router() -> Router {
    Router::new()
        // redirect to the oauth endpoint on gitlab
        .route("/login", axum::routing::get(login_get_endpoint))
        // the endpoint that gitlab will redirect into after successful login there
        .route(
            "/oauth/redirect",
            axum::routing::get(oauth_redirect_endpoint),
        )
        // logout an existing session
        .route("/logout", axum::routing::post(logout_post_endpoint))
}

#[derive(Debug, Deserialize)]
pub struct LoginQueryNext {
    next: Option<String>,
}

pub async fn login_get_endpoint(
    auth_session: AuthSession,
    session: Session,
    Query(next): Query<LoginQueryNext>,
) -> impl IntoResponse {
    let (auth_url, csrf_state, pkce_verifier) = auth_session.backend.authorize_url();

    session
        .insert(CSRF_STATE_KEY, csrf_state.secret())
        .await
        .expect("String serialization is infallible.");
    session
        .insert(PKCE_VERIFIER_KEY, pkce_verifier.secret())
        .await
        .expect("String serialization is infallible.");
    if let Some(next_location) = next.next {
        session
            .insert(NEXT_URL_KEY, next_location)
            .await
            .expect("String serialization is infallible.");
    }

    // insert next location in to the user session??
    axum::response::Redirect::to(auth_url.as_str()).into_response()
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    code: String,
    state: CsrfToken,
}

pub async fn oauth_redirect_endpoint(
    mut auth_session: AuthSession,
    session: Session,
    Query(AuthzResp {
        code,
        state: csrf_state,
    }): Query<AuthzResp>,
) -> impl IntoResponse {
    let Ok(Some(known_csrf_state)) = session.get(CSRF_STATE_KEY).await else {
        warn!("oauth redirect called with inconsistent csrf state");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let pkce_verifier = match session.get(PKCE_VERIFIER_KEY).await {
        // everything okay
        Ok(Some(x)) => x,
        // the pkce verifier was not set
        Ok(None) => {
            error!("The PKCE verifier was missing while dealing with an oauth redirect with valid CSRF state");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        // unable to get data from the session
        Err(e) => {
            error!("Unable to get Session data while dealing with an oauth redirect with valid CSRF state");
            error!("Original error: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let creds = Credentials {
        code,
        csrf_state,
        known_csrf_state,
        pkce_verifier,
    };

    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Got Oauth Redirect but csrf state was invalid.");
            return StatusCode::UNAUTHORIZED.into_response();
        }
        Err(e) => {
            error!("Tried to authenticate user in the oauth redirect, but got the following error: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(e) = auth_session.login(&user).await {
        error!(
            "Unable to login user {} after getting correct oauth redirect: {e}",
            user.username
        );
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Ok(Some(next)) = session.remove::<String>(NEXT_URL_KEY).await {
        axum::response::Redirect::to(&next).into_response()
    } else {
        axum::response::Redirect::to("/").into_response()
    }
}

pub async fn logout_post_endpoint(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(_) => axum::response::Redirect::to("/login").into_response(),
        Err(e) => {
            error!("Failed to logout a user: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
