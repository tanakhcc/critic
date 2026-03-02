//! Take an image, and return a list of baselines on that page.
//!
//! This requires a segmentation model for kraken to be present on-disk (and you need to pass its
//! path).

use std::path::{Path, PathBuf};

use critic_config::Config;
use critic_db::{BaselineTask, KrakenTask, get_model_for_page};
use pyo3::ffi::c_str;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use critic_shared::{Region, SegmentedPage};

use crate::{IndexError, KrakenBaseline, KrakenRegion, TextDirection};

/// Given an image and a segmentation model by file path, calculate the segmentation.
pub fn segment_image<P1: AsRef<Path>, P2: AsRef<Path>>(
    image_path: P1,
    model_path: P2,
    text_direction: TextDirection,
) -> Result<SegmentedPage, IndexError> {
    Python::attach(|py| {
        let code = c_str!(include_str!("./py/segment.py"));
        let segment = PyModule::from_code(py, code, c_str!("segment.py"), c_str!("segment"))
            .expect("static code");

        let args = (image_path.as_ref().to_str(), model_path.as_ref().to_str());
        let kwargs = PyDict::new(py);
        kwargs.set_item("text_direction", text_direction.to_string())?;

        let segmentation = segment
            .getattr("segment")
            .expect("static code")
            .call_method("segment", args, Some(&kwargs))?;
        let lines = segmentation.getattr("lines")?;
        let lines_as_vec = lines.extract::<Vec<KrakenBaseline>>()?;
        let regions_any = segmentation.getattr("regions")?;
        let regions: &Bound<'_, PyDict> = regions_any
            .cast()
            .map_err(|e| IndexError::Cast(e.to_string()))?;
        let region_text = regions
            .get_item("text")?
            .ok_or(IndexError::NoTextInRegion)?;
        let regions_as_vec = region_text.extract::<Vec<KrakenRegion>>()?;
        lines_and_regions_to_segmented_page(lines_as_vec, regions_as_vec)
    })
}

/// Assign each line to its region.
fn lines_and_regions_to_segmented_page(
    lines: Vec<KrakenBaseline>,
    regions: Vec<KrakenRegion>,
) -> Result<SegmentedPage, IndexError> {
    // for each line
    // find the region its center of gravity is closest to
    todo!()
}

/// Handle the task of baselining a single manuscript page, given in `task`.
///
/// This performs the baselining via kraken and writes the results into the DB.
pub async fn handle_baseline_task(config: &Config, task: &BaselineTask) -> Result<(), IndexError> {
    let model = get_model_for_page(
        &config.db,
        &task.page,
        critic_shared::ModelType::Segmentation,
    )
    .await?;
    let model_path: PathBuf = [
        &config.data_directory,
        &model.directory(),
        "original.mlmodel",
    ]
    .iter()
    .collect();
    let image_path: PathBuf = [
        &config.data_directory,
        &task.manuscript,
        &task.page,
        "original.webp",
    ]
    .iter()
    .collect();

    tracing::warn!(
        "Using hardcoded value for TextDirection. Please implement this properly (one direction set per language)"
    );
    let baselines = segment_image(image_path, model_path, TextDirection::HorizontalRL)?;

    // write the result to DB
    todo!();
    Ok(())
}
