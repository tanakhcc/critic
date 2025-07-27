//! Communicate with githubs api

use std::sync::Arc;

use reqwest::{header::USER_AGENT, StatusCode};

use crate::{auth::AuthenticatedUser, config::Config};

#[derive(Debug)]
pub enum GithubApiError {
    /// Reqwest had problems making the request itself
    Reqwest(reqwest::Error),
    /// a userrole was returned, but that does not exist
    UserRoleDoesNotExist(i32),
    /// The user is not a member of the group
    UserNotGroupMember(i32),
    /// The status code from githubs api was what we assumed
    BadStatusCode(StatusCode),
}
impl From<reqwest::Error> for GithubApiError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}
impl core::fmt::Display for GithubApiError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Reqwest(e) => {
                write!(f, "Unable to complet HTTP request:{e}")
            }
            Self::UserRoleDoesNotExist(role_code) => {
                write!(f, "Returned user role does not exist: {role_code}")
            }
            Self::UserNotGroupMember(id) => {
                write!(
                    f,
                    "The user with id {id} is not member of the main group in github."
                )
            }
            Self::BadStatusCode(code) => {
                write!(f, "Got the following status code: {code} from github API.")
            }
        }
    }
}
impl core::error::Error for GithubApiError {}

pub async fn user_is_member(
    config: Arc<Config>,
    user: &AuthenticatedUser,
) -> Result<bool, GithubApiError> {
    let encoded_group_name = urlencoding::encode(&config.github.org_name);
    let request_url = format!(
        "https://api.github.com/orgs/{}/members/{}",
        encoded_group_name, user.username
    );
    let response = reqwest::Client::new()
        .get(request_url)
        .header(USER_AGENT.as_str(), "axum-login") // See: https://docs.github.com/en/rest/overview/resources-in-the-rest-api?apiVersion=2022-11-28#user-agent-required
        .bearer_auth(user.access_token.clone())
        .send()
        .await?;

    match response.status() {
        StatusCode::NO_CONTENT => Ok(true),
        StatusCode::NOT_FOUND => Ok(false),
        c => Err(GithubApiError::BadStatusCode(c)),
    }
}
