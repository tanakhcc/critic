//! Types used for the authentication flow that are required for the database insertions

use axum_login::AuthUser;
use oauth2::TokenResponse;
use serde::Deserialize;

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

impl AuthUser for AuthenticatedUser {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.access_token.as_bytes()
    }
}
