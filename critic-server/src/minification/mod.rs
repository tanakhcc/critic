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
use rayon::prelude::*;

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
    OpenOriginal(std::io::Error),
    /// The original file format cannot be guessed
    GuessFormat(std::io::Error),
    /// Cannot decode the image
    Decode(image::ImageError),
    /// Cannot save the image
    Save(image::ImageError),
}
impl core::fmt::Display for MinificationError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::OpenOriginal(e) => {
                write!(f, "The original page image can not be opened: {e}.")
            }
            Self::GuessFormat(e) => {
                write!(f, "Cannot guess format of the original file: {e}.")
            }
            Self::Decode(e) => {
                write!(f, "Cannot decode the image: {e}.")
            }
            Self::Save(e) => {
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
        .map_err(MinificationError::OpenOriginal)?
        .with_guessed_format()
        .map_err(MinificationError::GuessFormat)?
        .decode()
        .map_err(MinificationError::Decode)?;

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
        .map_err(MinificationError::Save)?;
    tracing::trace!(
        "Saving page {} of ms {msname} as webp in original dimensions",
        page.name
    );
    img.save(format!("{base_path}/original.webp"))
        .map_err(MinificationError::Save)?;

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
        let wait_till_next_minification = match get_page_to_minify(
            &config.db,
            config.worker_threads,
        )
        .await
        {
            Ok(pages) => {
                if pages.is_empty() {
                    // no page to minify or error getting one - try again later
                    tokio::time::Duration::from_secs(1)
                } else {
                    let config_arc = config.clone();
                    // attempt the minifications in parallel, without blocking this thread
                    let minify_results: Vec<(Result<(), MinificationError>, String, PageMeta)> =
                        tokio::task::spawn_blocking(move || {
                            pages
                                .into_par_iter()
                                .map(|(msname, page_to_minify)| {
                                    (
                                        minify_page(
                                            &config_arc.data_directory,
                                            &msname,
                                            &page_to_minify,
                                        ),
                                        msname,
                                        page_to_minify,
                                    )
                                })
                                .collect::<Vec<_>>()
                        })
                        .await
                        .unwrap();
                    for (res, msname, page) in minify_results {
                        match res {
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to minify page {} of ms {msname}: {e}",
                                    page.name,
                                );
                                if let Err(e) =
                                    mark_page_minifcation_failed(&config.db, page.id).await
                                {
                                    tracing::warn!(
                                            "Failed to mark page {} of ms {msname} minification as failed: {e}",
                                            page.name
                                        );
                                };
                            }
                            Ok(()) => {
                                // finally, mark the page as minified
                                if let Err(e) = mark_page_minified(&config.db, page.id).await {
                                    tracing::warn!("Failed marking page {} of ms {msname} as minified, but minification is done: {e}", page.name)
                                };
                            }
                        }
                    }
                    tokio::time::Duration::from_millis(10)
                }
            }
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
