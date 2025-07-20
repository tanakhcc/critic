//! Endpoints for uploading stuff to the server

use std::sync::Arc;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path},
    response::IntoResponse,
    Extension, Json,
};
use critic_shared::{
    urls::IMAGE_BASE_LOCATION, FileTransferResponse, ALLOWED_IMAGE_EXTENSIONS, MAX_BODY_SIZE,
};
use reqwest::StatusCode;

use crate::{
    auth::AuthSession,
    config::Config,
    db::add_page,
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
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
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
            let mut results = FileTransferResponse::new();
            loop {
                match mpart.next_field().await {
                    Ok(Some(field)) => {
                        let Some(file_name) = field.file_name() else {
                            results
                                .push_err("The file name must be set for each file.".to_string());
                            continue;
                        };
                        let mut dot_split = file_name.split('.');
                        let base_name = match dot_split.next() {
                            Some(x) => x.to_string(),
                            None => {
                                results
                                    .push_err("Filename did not contain a basename.".to_string());
                                continue;
                            }
                        };
                        let extension = match dot_split.next() {
                            Some(x) => x.to_string(),
                            None => {
                                results
                                    .push_err("Filename did not contain an extension.".to_string());
                                continue;
                            }
                        };
                        if !ALLOWED_IMAGE_EXTENSIONS.contains(&extension.as_str()) {
                            results.push_err("Extension is not allowed.".to_string());
                            continue;
                        };
                        if dot_split.next().is_some() {
                            results
                                .push_err("Filename did not contain exactly one dot.".to_string());
                            continue;
                        };

                        let data = field.bytes().await.unwrap();

                        // try insert into the DB first
                        if let Err(e) = add_page(&config.db, &base_name, &msname).await {
                            tracing::warn!("Failed to insert new page {base_name} for {msname} into the db: {e}");
                            results
                                .push_err(format!("Failed to insert new page into the db: {e}."));
                            continue;
                        }
                        // that worked - now deal with the file system
                        let directory_path = format!(
                            "{}{}/{msname}/{base_name}",
                            config.data_directory, IMAGE_BASE_LOCATION
                        );
                        if let Err(e) = std::fs::create_dir_all(&directory_path) {
                            results.push_err(format!(
                                "Failed to crate directory to put new page into: {e}."
                            ));
                            continue;
                        };
                        if let Err(e) = std::fs::write(format!("{directory_path}/original"), data) {
                            tracing::warn!("Unable to write manuscript page to file: {e}");
                            results.push_err("Failed to write Page to file.".to_string());
                            continue;
                        }
                        tracing::info!(
                            "{} saved new page for {msname}: {base_name}.{extension}.",
                            user.username
                        );
                        results.push_ok();
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
            (
                if results.err.iter().all(|e| e.is_none()) {
                    StatusCode::OK
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                },
                Json(results),
            )
                .into_response()
        }
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}
