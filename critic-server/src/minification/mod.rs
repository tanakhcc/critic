//! Minification for images
//!
//! Each image is stored in its proper directory as `original`
//! Minification is set to `false` in the db by default
//!
//! The minifier then goes over files in the db with minification set to false and tries to minify
//! them
//!
//! Rescaled images are converted to webp:
//! - at the original size (just convert so we can show images as webp)
//! - at preview scale

use std::sync::Arc;

use crate::config::Config;

/// Dimensions for images in the preview downscale
const PREVIEW_IMAGE_DIMENSIONS: (i32, i32) = (720, 960);

pub async fn minification_service(config: Arc<Config>) -> Result<(), Box<dyn core::error::Error>> {
    Ok(())
}

