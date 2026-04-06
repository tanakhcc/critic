//! Take an image, and return a list of baselines on that page.
//!
//! This requires a segmentation model for kraken to be present on-disk (and you need to pass its
//! path).

use std::path::{Path, PathBuf};

use critic_config::Config;
use critic_db::{
    BaselineTask, KrakenTask, get_language_for_page, get_model_for_page, insert_segmentation,
};
use geo::Centroid;
use pyo3::ffi::c_str;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use critic_shared::{
    Baseline, Point, Region, SegmentedPage, TextDirection, urls::IMAGE_BASE_LOCATION,
};

use crate::{IndexError, KrakenBaseline, KrakenRegion};

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

        let segmentation = segment.call_method("segment", args, Some(&kwargs))?;
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
    ocr_lines: Vec<KrakenBaseline>,
    ocr_regions: Vec<KrakenRegion>,
) -> Result<SegmentedPage, IndexError> {
    // for each line
    // find the region its center of gravity is closest to
    let mut regions = ocr_regions
        .into_iter()
        .map(core::convert::TryInto::try_into)
        .collect::<Result<Vec<_>, ()>>()
        .map_err(|()| IndexError::RegionFormat)?;
    // get the regions centroid
    let centroids = regions
        .iter()
        .map(|r: &Region| {
            let as_geo_poly = geo::Polygon::new(
                r.boundary
                    .points
                    .iter()
                    .map(|p| geo::coord! {x: p.x as f32, y: p.y as f32})
                    .collect(),
                Vec::with_capacity(0),
            );
            as_geo_poly.centroid()
        })
        .collect::<Vec<_>>();
    for ocr_line in ocr_lines {
        // get the lines centroid
        let line: Baseline = ocr_line
            .try_into()
            .map_err(|()| IndexError::BaselineFormat)?;
        let line_centroid = line.centroid();
        let Some(closest_region_idx) = centroids
            .iter()
            .flatten()
            .map(|centroid| {
                (centroid.x() - line_centroid.x as f32).powi(2)
                    + (centroid.y() - line_centroid.y as f32).powi(2)
            })
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
        else {
            continue;
        };
        regions[closest_region_idx].baselines.push(line);
    }
    Ok(SegmentedPage { regions })
}

/// Handle the task of baselining a single manuscript page, given in `task`.
///
/// This performs the baselining via kraken and writes the results into the DB.
pub async fn handle_baseline_task(config: &Config, task: &BaselineTask) -> Result<(), IndexError> {
    let Some(model) = get_model_for_page(
        &config.db,
        &task.page,
        critic_shared::ModelType::Segmentation,
    )
    .await?
    else {
        return Err(IndexError::NoSegmentationModel(
            task.manuscript.clone(),
            task.page.clone(),
        ));
    };

    let model_path: PathBuf = [
        &config.data_directory,
        &model.directory(),
        "original.mlmodel",
    ]
    .iter()
    .collect();

    let image_path: PathBuf = [
        &config.data_directory,
        &IMAGE_BASE_LOCATION[1..],
        &task.manuscript,
        &task.page,
        "original.webp",
    ]
    .iter()
    .collect();

    let language = get_language_for_page(&config.db, &task.page).await?;

    tracing::trace!("Now segmenting image {image_path:?}.");
    let image_path_for_spawn = image_path.clone();
    let segmentation = tokio::task::spawn_blocking(move || {
        segment_image(image_path_for_spawn, model_path, language.text_direction)
    })
    .await??;
    insert_segmentation(&config.db, &task.manuscript, &task.page, &segmentation).await?;
    tracing::trace!("Finished segmenting image {image_path:?}.");
    Ok(())
}
