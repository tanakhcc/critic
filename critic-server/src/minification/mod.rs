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

use std::{fs::remove_file, sync::Arc};

use critic_shared::{urls::IMAGE_BASE_LOCATION, PageMeta};
use image::{imageops::resize, GenericImageView, ImageReader};

use crate::{
    config::Config,
    db::{get_page_to_minify, mark_page_minifcation_failed, mark_page_minified},
    signal_handler::InShutdown,
};

/// width of the preview downcales for manuscript pages
/// the height will be calculated to keep the same aspect ratio
const PREVIEW_IMAGE_WIDTH: u32 = 720;

/// Problems that can occur during minification
#[derive(Debug)]
enum MinificationError {
    /// The original file cannot be opened
    CannotOpenOriginal(std::io::Error),
    /// The original file format cannot be guessed
    CannotGuessFormat(std::io::Error),
    /// Cannot decode the image
    CannotDecode(image::ImageError),
    /// Cannot save the image
    CannotSave(image::ImageError),
}
impl core::fmt::Display for MinificationError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::CannotOpenOriginal(e) => {
                write!(f, "The original page image can not be opened: {e}.")
            }
            Self::CannotGuessFormat(e) => {
                write!(f, "Cannot guess format of the original file: {e}.")
            }
            Self::CannotDecode(e) => {
                write!(f, "Cannot decode the image: {e}.")
            }
            Self::CannotSave(e) => {
                write!(f, "Cannot save the image: {e}.")
            }
        }
    }
}
impl core::error::Error for MinificationError {}

/// Minify a single page, blocking the thread during resizing/reading/...
fn minify_page(
    data_directory: &str,
    msname: &str,
    page: &PageMeta,
) -> Result<(), MinificationError> {
    tracing::trace!(
        "Start minification for a new page: {} of ms {msname}",
        page.name
    );
    let base_path = format!(
        "{data_directory}{IMAGE_BASE_LOCATION}/{msname}/{}",
        page.name
    );
    let img = ImageReader::open(format!("{base_path}/original"))
        .map_err(MinificationError::CannotOpenOriginal)?
        .with_guessed_format()
        .map_err(MinificationError::CannotGuessFormat)?
        .decode()
        .map_err(MinificationError::CannotDecode)?;

    // keep aspect ratio of the image
    let target_height = PREVIEW_IMAGE_WIDTH * img.dimensions().1 / img.dimensions().0;
    tracing::trace!("Start resizing page: {} of ms {msname}", page.name);
    let resized = resize(
        &img,
        PREVIEW_IMAGE_WIDTH,
        target_height,
        image::imageops::FilterType::Lanczos3,
    );
    tracing::trace!("Saving Preview for page: {} of ms {msname}", page.name);
    resized
        .save(format!("{base_path}/preview.webp"))
        .map_err(MinificationError::CannotSave)?;
    tracing::trace!(
        "Saving page {} of ms {msname} as webp in original dimensions",
        page.name
    );
    img.save(format!("{base_path}/original.webp"))
        .map_err(MinificationError::CannotSave)?;

    // now delete the original, we only care about the webp version
    tracing::trace!(
        "Deleting non-webp original for {} of ms {msname}",
        page.name
    );
    if let Err(e) = remove_file(format!("{base_path}/original")) {
        tracing::warn!("Failed to unlink original ms page file: {base_path}/original : {e}. Will not retry and leave the file orphaned.");
    };
    Ok(())
}

/// Run the minification service
pub async fn run_minification(
    config: Arc<Config>,
    mut watcher: tokio::sync::watch::Receiver<InShutdown>,
) {
    tracing::debug!("Starting the minification service");
    loop {
        let wait_till_next_minification = match get_page_to_minify(&config.db).await {
            Ok(Some((msname, page_to_minify))) => {
                // minify, directly continue with the next one if that worked
                match minify_page(&config.data_directory, &msname, &page_to_minify) {
                    Err(e) => {
                        tracing::warn!(
                            "Failed to minify page {} of ms {msname}: {e}",
                            page_to_minify.name,
                        );
                        if let Err(e) =
                            mark_page_minifcation_failed(&config.db, page_to_minify.id).await
                        {
                            tracing::warn!(
                                "Failed to mark page {} of ms {msname} minification as failed: {e}",
                                page_to_minify.name
                            );
                        };
                    }
                    Ok(()) => {
                        // finally, mark the page as minified
                        if let Err(e) = mark_page_minified(&config.db, page_to_minify.id).await {
                            tracing::warn!("Failed marking page {} of ms {msname} as minified, but minification is done: {e}", page_to_minify.name)
                        };
                    }
                };
                tokio::time::Duration::from_millis(10)
            }
            // no page to minify or error getting one - try again later
            Ok(None) => tokio::time::Duration::from_secs(1),
            Err(e) => {
                tracing::warn!("Failed to get page to minify: {e}");
                // this may be a general problem with the DB, so we do not want to bombard it with
                // useless requests
                tokio::time::Duration::from_secs(5)
            }
        };
        // now wait a bit, or cancel the service if we are in shutdown
        tokio::select! {
            _ = watcher.changed() => {
                tracing::debug!("Shutting down minification service now.");
                return;
            }
            _ = tokio::time::sleep(wait_till_next_minification) => {}
        };
    }
}
