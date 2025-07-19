//! Endpoints for uploading stuff to the server

use std::sync::Arc;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path},
    response::IntoResponse,
    Extension, Json,
};
use critic_shared::{urls::IMAGE_BASE_LOCATION, FileTransferOkResponse};
use reqwest::StatusCode;

use crate::{
    auth::AuthSession,
    config::Config,
    gitlab::{get_user_role, GitlabUserRole},
};

/// The router handling all file uploads
pub fn upload_router() -> axum::Router {
    axum::Router::new()
        .route(
            &format!(
                "{}/{{msname}}",
                critic_shared::urls::PAGE_UPLOAD_API_ENDPOINT
            ),
            axum::routing::post(page_upload),
        )
        .layer(DefaultBodyLimit::max(1024 * 1024 * 100))
}

/// Upload several pages for a manuscript
pub async fn page_upload(
    Extension(config): Extension<Arc<Config>>,
    Path(msname): Path<String>,
    auth_session: AuthSession,
    mut mpart: Multipart,
) -> impl IntoResponse {
    if let Some(user) = auth_session.user {
        let user_role = match get_user_role(config.clone(), &user).await {
            Ok(x) => x,
            Err(e) => {
                tracing::warn!("Unable to get the user role for {}: {e}", user.username);
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        };
        if user_role < GitlabUserRole::Maintainer {
            StatusCode::UNAUTHORIZED.into_response()
        } else {
            // now iterate over the different files and save them
            let mut new_pages = 0;
            loop {
                match mpart.next_field().await {
                    Ok(Some(field)) => {
                        let Some(file_name) = field.file_name() else {
                            return (
                                StatusCode::BAD_REQUEST,
                                "The file name must be set for each file.",
                            )
                                .into_response();
                        };
                        let mut dot_split = file_name.split('.');
                        let base_name = match dot_split.next() {
                            Some(x) => x.to_string(),
                            None => {
                                return (
                                    StatusCode::BAD_REQUEST,
                                    "Filename did not contain a basename.",
                                )
                                    .into_response();
                            }
                        };
                        let extension = match dot_split.next() {
                            Some(x) => x.to_string(),
                            None => {
                                return (
                                    StatusCode::BAD_REQUEST,
                                    "Filename did not contain an extension.",
                                )
                                    .into_response();
                            }
                        };
                        if dot_split.next().is_some() {
                            return (
                                StatusCode::BAD_REQUEST,
                                "Filename did not contain exactly one dot.",
                            )
                                .into_response();
                        };

                        let data = field.bytes().await.unwrap();

                        let directory_path = format!(
                            "{}{}/{msname}/{base_name}",
                            config.data_directory, IMAGE_BASE_LOCATION
                        );
                        if let Err(e) = std::fs::create_dir_all(&directory_path) {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to crate directory to put new page into: {e}."),
                            )
                                .into_response();
                        };
                        match std::fs::write(format!("{directory_path}/original.{extension}"), data)
                        {
                            Ok(()) => {
                                // TODO
                                // start minifcation in new thread
                                // add database entry
                                tracing::debug!(
                                    "saved new page for {msname} to file: {base_name}.{extension}."
                                );
                                new_pages += 1;
                            }
                            Err(e) => {
                                tracing::warn!("Unable to write manuscript page to file: {e}");
                                return (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    "Failed to write Page to file.",
                                )
                                    .into_response();
                            }
                        }
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(e) => {
                        tracing::warn!("Failed reading one of the multipart fields: {e}");
                        tracing::warn!("logged in user: {}", user.username);
                    }
                };
            }
            (StatusCode::OK, Json(FileTransferOkResponse { new_pages })).into_response()
        }
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}
