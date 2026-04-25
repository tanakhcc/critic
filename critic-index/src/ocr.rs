//! Turning an image and baselines into recognized text.

use std::path::{Path, PathBuf};

use critic_config::Config;
use critic_db::{OcrTask, get_language_for_page, get_model_for_page, get_segmentation, update_ocr};
use critic_shared::urls::IMAGE_BASE_LOCATION;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyo3::{ffi::c_str, types::PyList};

use critic_shared::{Baseline, Point, TextDirection};
use tantivy::Searcher;

use crate::fts::basetext_from_proposal;
use crate::{IndexError, OcrRecord};

/// Given an image and a segmentation model by file path, calculate the segmentation.
pub fn ocr_image<'a, P1: AsRef<Path>, P2: AsRef<Path>>(
    image_path: P1,
    model_path: P2,
    text_direction: TextDirection,
    baselines: Vec<Baseline>,
    equality_alphabet: Option<String>,
) -> Result<Vec<OcrRecord>, IndexError> {
    Python::attach(|py| {
        let code = c_str!(include_str!("./py/ocr.py"));
        let ocr =
            PyModule::from_code(py, code, c_str!("ocr.py"), c_str!("ocr")).expect("static code");

        // gets the python type
        // list[tuple[baseline: tuple[tuple[x, y]], boundary: list[tuple[x, y]]]]
        let baselines_and_boundaries: Vec<_> = baselines
            .into_iter()
            .map(|bl| {
                (
                    (
                        (bl.baseline.0.x, bl.baseline.0.y),
                        (bl.baseline.1.x, bl.baseline.1.y),
                    ),
                    bl.boundary
                        .points
                        .iter()
                        .map(|p| (p.x as f64, p.y as f64))
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        let args = (
            image_path.as_ref().to_str(),
            model_path.as_ref().to_str(),
            baselines_and_boundaries,
        );
        let kwargs = PyDict::new(py);
        kwargs.set_item("text_direction", text_direction.to_string())?;

        let ocr = ocr.call_method("ocr", args, Some(&kwargs))?;
        let records: &Bound<'_, PyList> =
            ocr.cast().map_err(|e| IndexError::Cast(e.to_string()))?;
        records
            .into_iter()
            .map(|record| {
                let as_tuple: &Bound<'_, PyTuple> =
                    record.cast().map_err(|e| IndexError::Cast(e.to_string()))?;
                let prediction_as_str: String = as_tuple
                    .get_item(0)
                    .map_err(|e| IndexError::Cast(e.to_string()))?
                    .extract()
                    .map_err(|e| IndexError::Cast(e.to_string()))?;
                let baseline_as_any = as_tuple
                    .get_item(1)
                    .map_err(|e| IndexError::Cast(e.to_string()))?;
                let baseline_start: (u32, u32) = baseline_as_any
                    .get_item(0)
                    .map_err(|e| IndexError::Cast(e.to_string()))?
                    .extract()
                    .map_err(|e| IndexError::Cast(e.to_string()))?;
                let baseline_end: (u32, u32) = baseline_as_any
                    .get_item(-1)
                    .map_err(|e| IndexError::Cast(e.to_string()))?
                    .extract()
                    .map_err(|e| IndexError::Cast(e.to_string()))?;
                let typed_ocr_record = OcrRecord::new(
                    &prediction_as_str,
                    (Point::from(baseline_start), Point::from(baseline_end)),
                    equality_alphabet.as_deref(),
                );
                Ok(typed_ocr_record)
            })
            .collect()
    })
}

/// Handle a single OCR task.
///
/// We get a page and are tasked to create a best-guess basetext for this page.
pub async fn handle_ocr_task(
    config: &Config,
    task: &OcrTask,
    searcher: Searcher,
) -> Result<(), IndexError> {
    let Some(model) = get_model_for_page(
        &config.db,
        &task.page,
        critic_shared::ModelType::Recognition,
    )
    .await?
    else {
        return Err(IndexError::NoOcrModel(
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

    // get the baselines from the DB
    let mut segmentation = get_segmentation(&config.db, &task.manuscript, &task.page).await?;
    let language = get_language_for_page(&config.db, &task.manuscript, &task.page).await?;

    // get the OCR result from kraken
    tracing::trace!("Now running OCR on image {image_path:?}.");
    let baselines = segmentation
        .regions
        .iter()
        .map(|r| r.baselines.clone())
        .flatten()
        .collect();
    let image_path_for_spawn = image_path.clone();
    let equality_alphabet_for_spawn = language.equality_alphabet.clone();
    let ocr = tokio::task::spawn_blocking(move || {
        ocr_image(
            image_path_for_spawn,
            model_path,
            language.text_direction,
            baselines,
            equality_alphabet_for_spawn,
        )
    })
    .await??;

    tracing::trace!(
        "Finished OCR on image {image_path:?}. Now identifying the result in the base corpus."
    );

    // call to the indexing machine to find the correct basetext from the proposed OCR text
    let indexed_basetext = basetext_from_proposal(
        config,
        searcher,
        &ocr,
        &language.name,
        language.equality_alphabet.as_deref(),
    )
    .await?;
    segmentation.insert_basetext_into_segmentation(indexed_basetext);
    update_ocr(&config.db, &task.page, &segmentation, true).await?;
    tracing::debug!("Finished OCR task on {image_path:?} and inserted the result in the DB.");
    Ok(())
}
