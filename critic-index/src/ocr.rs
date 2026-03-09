//! Turning an image and baselines into recognized text.

use std::path::{Path, PathBuf};

use critic_config::Config;
use critic_db::{OcrTask, get_model_for_page};
use critic_shared::urls::IMAGE_BASE_LOCATION;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyo3::{ffi::c_str, types::PyList};

use critic_shared::{Point, TextDirection};
use tantivy::Searcher;

use crate::{IndexError, OcrRecord};

/// Given an image and a segmentation model by file path, calculate the segmentation.
pub fn ocr_image<P1: AsRef<Path>, P2: AsRef<Path>>(
    image_path: P1,
    model_path: P2,
    text_direction: TextDirection,
    baselines: Vec<((i32, i32), (i32, i32))>,
) -> Result<Vec<OcrRecord>, IndexError> {
    Python::attach(|py| {
        let code = c_str!(include_str!("./py/ocr.py"));
        let ocr =
            PyModule::from_code(py, code, c_str!("ocr.py"), c_str!("ocr")).expect("static code");

        let args = (
            image_path.as_ref().to_str(),
            model_path.as_ref().to_str(),
            baselines,
        );
        let kwargs = PyDict::new(py);
        kwargs.set_item("text_direction", text_direction.to_string())?;

        tracing::trace!("Starting OCR for {:?}", image_path.as_ref());
        let ocr =
            ocr.getattr("ocr")
                .expect("static code")
                .call_method("ocr", args, Some(&kwargs))?;
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
                let typed_ocr_record = OcrRecord {
                    prediction: prediction_as_str,
                    baseline: (Point::from(baseline_start), Point::from(baseline_end)),
                };
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

    // get the OCR result from kraken

    // call to the indexing machine to find the correct basetext from the proposed OCR text
    //          this function actually needs to live in the continuous indexer and get the searcher
    //          from there
    // write the result to DB

    todo!()
}
