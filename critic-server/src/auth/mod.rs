//! All types and endpoints for authenticating users

use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum_login::{AuthUser, AuthnBackend, UserId};
use oauth2::{
    url::Url, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope,
    TokenResponse,
};
use serde::Deserialize;

use crate::{
    config::Config,
    db::{self, DBError},
};

// some basic types used across the app
/// The JSON object returned from githubs get-user endpoint
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    /// ID of the user in github - we use the same ID in the internal DB here
    pub id: i32,
    /// username of the user in github - we use the same here
    pub login: String,
}
impl From<AuthenticatedUser> for UserInfo {
    fn from(value: AuthenticatedUser) -> Self {
        Self {
            id: value.id,
            login: value.username,
        }
    }
}

/// The full User with oauth2 credentials
#[derive(Deserialize, Clone, sqlx::prelude::FromRow)]
pub struct AuthenticatedUser {
    pub id: i32,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: time::OffsetDateTime,
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
pub enum NormalizeTokenResponseError {
    NoRefresh,
    NoExpiresIn,
}
impl core::fmt::Display for NormalizeTokenResponseError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::NoRefresh => {
                write!(f, "No refresh token was given")
            }
            Self::NoExpiresIn => {
                write!(f, "No expires_in time was given")
            }
        }
    }
}
impl std::error::Error for NormalizeTokenResponseError {}
#[derive(Debug)]
pub struct NormalizedTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: time::OffsetDateTime,
}
impl
    TryFrom<
        oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    > for NormalizedTokenResponse
{
    type Error = NormalizeTokenResponseError;

    fn try_from(
        value: oauth2::StandardTokenResponse<
            oauth2::EmptyExtraTokenFields,
            oauth2::basic::BasicTokenType,
        >,
    ) -> Result<Self, Self::Error> {
        let expires_at = time::OffsetDateTime::now_utc()
            + value
                .expires_in()
                .ok_or(NormalizeTokenResponseError::NoExpiresIn)?;
        Ok(Self {
            access_token: value.access_token().clone().into_secret(),
            refresh_token: value
                .refresh_token()
                .ok_or(NormalizeTokenResponseError::NoRefresh)?
                .clone()
                .into_secret(),
            expires_at,
        })
    }
}

/// has all the backend APIs for auth flows
pub mod backend;

impl AuthUser for AuthenticatedUser {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.access_token.as_bytes()
    }
}

/// Known secrets for this Oauth2 Flow before getting authorization_token
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    /// the authorization code returned from the oauth server
    code: String,
    /// the CSRF state returned from the oauth server
    csrf_state: CsrfToken,
    /// the CSRF state known from when we created this request
    known_csrf_state: CsrfToken,
    /// the pkce code verifier known from when we created this request
    pkce_verifier: String,
}

/// The types of Problems that can occur while doing an oauth2 flow
#[derive(Debug)]
pub enum BackendError {
    /// failure while talking to our postgres
    DB(DBError),
    /// failure while calling the /oauth/token endpoint in github - could not get token
    TokenExchange(String),
    Reqwest(reqwest::Error),
    Github(reqwest::Error),
    TokenResponse(NormalizeTokenResponseError),
}
impl core::fmt::Display for BackendError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::DB(e) => {
                write!(f, "Failure while talking to our own database: {e}")
            }
            Self::TokenExchange(e) => {
                write!(
                    f,
                    "Failure while exchanging authorization code for access token: {e}"
                )
            }
            Self::Reqwest(e) => {
                write!(f, "Failure sending http request: {e}")
            }
            Self::Github(e) => {
                write!(f, "Failure to parse response JSON from github API: {e}")
            }
            Self::TokenResponse(e) => {
                write!(
                    f,
                    "Token response from githubs api was not as expected: {e}"
                )
            }
        }
    }
}
impl std::error::Error for BackendError {}

#[derive(Debug, Clone)]
pub struct GithubOauthBackend {
    db: sqlx::Pool<sqlx::Postgres>,
    client: crate::config::OauthClient,
}

impl GithubOauthBackend {
    pub fn new(config: std::sync::Arc<Config>) -> Self {
        let db = config.db.clone();
        let client = config.oauth_client.clone();
        Self { db, client }
    }

    /// URL to show to the user to start the oauth flow
    /// RETURNS
    ///     the url to show
    ///     the CsrfToken in use
    pub fn authorize_url(&self) -> (Url, CsrfToken, PkceCodeVerifier) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("api".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        (url, csrf_token, pkce_verifier)
    }
}

#[async_trait::async_trait]
impl AuthnBackend for GithubOauthBackend {
    type User = AuthenticatedUser;
    type Credentials = Credentials;
    type Error = BackendError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        // Ensure the CSRF state has not been tampered with.
        if creds.known_csrf_state.secret() != creds.csrf_state.secret() {
            return Ok(None);
        };

        // Process authorization code, expecting a token response back.
        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("static client");
        let token_res = self
            .client
            // authorization code is known from session
            .exchange_code(AuthorizationCode::new(creds.code))
            // PKCE code verifier is known from session
            .set_pkce_verifier(PkceCodeVerifier::new(creds.pkce_verifier))
            .request_async(&client)
            .await
            .map_err(|e| BackendError::TokenExchange(e.to_string()))?;

        // Use access token to request user info.
        let user_info = client
            .get("https://api.github.com/user")
            .header(USER_AGENT.as_str(), "axum-login") // See: https://docs.github.com/en/rest/overview/resources-in-the-rest-api?apiVersion=2022-11-28#user-agent-required
            .header(
                AUTHORIZATION.as_str(),
                format!("Bearer {}", token_res.access_token().secret()),
            )
            .send()
            .await
            .map_err(Self::Error::Reqwest)?;
        let user_info = user_info
            .json::<UserInfo>()
            .await
            .map_err(Self::Error::Github)?;

        // Persist user in our database so we can use `get_user`.
        let user = db::insert_or_update_user_session(
            &self.db,
            user_info,
            token_res.try_into().map_err(BackendError::TokenResponse)?,
        )
        .await
        .map_err(BackendError::DB)?;
        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        sqlx::query_as!(
            AuthenticatedUser,
            "select * from user_session where id = $1",
            user_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| Self::Error::DB(DBError::CannotGetUsersession(e)))
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<GithubOauthBackend>;
