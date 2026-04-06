//! Static content serving
//!
//! This includes:
//! - images

use axum::routing::get_service;
use critic_shared::urls::{
    FTS_INDEX_BASE_LOCATION, IMAGE_BASE_LOCATION, MODEL_BASE_LOCATION, TRANSCRIPTION_BASE_LOCATION,
};
use tower_http::services::ServeDir;

/// Creates the following directory structure if it does not exist
/// <data_directory>
///     /files
/// If any of the intermediate paths already exist as files, this fails
fn create_data_directory_layout(data_directory: &str) -> Result<(), std::io::Error> {
    // the directory for manuscript images
    std::fs::create_dir_all(format!("{data_directory}{IMAGE_BASE_LOCATION}"))?;
    std::fs::create_dir_all(format!("{data_directory}{TRANSCRIPTION_BASE_LOCATION}"))?;
    std::fs::create_dir_all(format!("{data_directory}{MODEL_BASE_LOCATION}"))?;
    std::fs::create_dir_all(format!("{data_directory}{FTS_INDEX_BASE_LOCATION}"))?;
    Ok(())
}

pub fn image_dir_router(data_directory: &str) -> Result<axum::Router, std::io::Error> {
    // create the data directory if it does not exist
    if let Err(e) = create_data_directory_layout(data_directory) {
        tracing::error!("Failed to create data directory layout: {e}");
        return Err(e);
    };
    tracing::debug!("Data directory layout is correct.");
    Ok(axum::Router::new().nest_service(
        IMAGE_BASE_LOCATION,
        get_service(ServeDir::new(format!(
            "{data_directory}{IMAGE_BASE_LOCATION}"
        ))),
    ))
}

pub fn get_image_dimensions(
    data_directory: &str,
    msname: String,
    pagename: String,
    which: critic_shared::ImageType,
) -> Result<(u32, u32), image::ImageError> {
    let path = format!("{data_directory}{IMAGE_BASE_LOCATION}/{msname}/{pagename}/{which}.webp");
    image::image_dimensions(path)
}
