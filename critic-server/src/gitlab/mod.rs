//! Communicate with gitlabs api

use std::{cmp::Ordering, sync::Arc};

use reqwest::StatusCode;
use serde::Deserialize;

use crate::{auth::AuthenticatedUser, config::Config};

/// The base URL in gitlab we communicate with - directly after the server name
const API_BASE_URL: &str = "/api/v4";

#[derive(Debug)]
pub enum GitlabApiError {
    /// Reqwest had problems making the request itself
    Reqwest(reqwest::Error),
    /// a userrole was returned, but that does not exist
    UserRoleDoesNotExist(i32),
    /// The user is not a member of the group
    UserNotGroupMember(i32),
    /// The status code from gitlabs api was what we assumed
    BadStatusCode(StatusCode),
}
impl From<reqwest::Error> for GitlabApiError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}
impl core::fmt::Display for GitlabApiError {
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
                    "The user with id {id} is not member of the main group in gitlab."
                )
            }
            Self::BadStatusCode(code) => {
                write!(f, "Got the following status code: {code} from gitlab API.")
            }
        }
    }
}
impl core::error::Error for GitlabApiError {}

pub enum GitlabUserRole {
    NoAccess,
    Minimal,
    Guest,
    Planner,
    Reporter,
    Developer,
    Maintainer,
    Owner,
    Admin,
}
impl TryFrom<i32> for GitlabUserRole {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GitlabUserRole::NoAccess),
            5 => Ok(GitlabUserRole::Minimal),
            10 => Ok(GitlabUserRole::Guest),
            15 => Ok(GitlabUserRole::Planner),
            20 => Ok(GitlabUserRole::Reporter),
            30 => Ok(GitlabUserRole::Developer),
            40 => Ok(GitlabUserRole::Maintainer),
            50 => Ok(GitlabUserRole::Owner),
            60 => Ok(GitlabUserRole::Admin),
            _ => Err(()),
        }
    }
}
impl From<&GitlabUserRole> for i32 {
    fn from(value: &GitlabUserRole) -> Self {
        match value {
            GitlabUserRole::NoAccess => 0,
            GitlabUserRole::Minimal => 5,
            GitlabUserRole::Guest => 10,
            GitlabUserRole::Planner => 15,
            GitlabUserRole::Reporter => 20,
            GitlabUserRole::Developer => 30,
            GitlabUserRole::Maintainer => 40,
            GitlabUserRole::Owner => 50,
            GitlabUserRole::Admin => 60,
        }
    }
}
impl PartialEq for GitlabUserRole {
    fn eq(&self, other: &GitlabUserRole) -> bool {
        Into::<i32>::into(self) == Into::<i32>::into(other)
    }
}
impl Eq for GitlabUserRole {}
impl PartialOrd for GitlabUserRole {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for GitlabUserRole {
    fn cmp(&self, other: &GitlabUserRole) -> Ordering {
        Into::<i32>::into(self).cmp(&other.into())
    }
}

#[derive(Deserialize)]
struct GroupMember {
    access_level: i32,
}

pub async fn get_user_role(
    config: Arc<Config>,
    user: &AuthenticatedUser,
) -> Result<GitlabUserRole, GitlabApiError> {
    let encoded_group_name = urlencoding::encode(&config.gitlab.group_name);
    let request_url = format!(
        "https://{}/{API_BASE_URL}/groups/{}/members/{}",
        config.gitlab.addr, encoded_group_name, user.id
    );
    let response = reqwest::Client::new()
        .get(request_url)
        .bearer_auth(user.access_token.clone())
        .send()
        .await?;
    match response.status() {
        StatusCode::NOT_FOUND => Err(GitlabApiError::UserNotGroupMember(user.id)),
        StatusCode::OK => {
            let response = response.json::<GroupMember>().await?;
            Ok(response
                .access_level
                .try_into()
                .map_err(|_| GitlabApiError::UserRoleDoesNotExist(response.access_level))?)
        }
        c => Err(GitlabApiError::BadStatusCode(c)),
    }
}
