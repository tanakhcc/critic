//! Take an image, and return a list of baselines on that page.
//!
//! This requires a segmentation model for kraken to be present on-disk (and you need to pass its
//! path).

use std::path::Path;

use pyo3::ffi::c_str;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use critic_shared::{Region, SegmentedPage};

use crate::IndexError;

pub enum TextDirection {
    HorizontalRL,
    HorizontalLR,
}
impl core::fmt::Display for TextDirection {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::HorizontalRL => {
                write!(f, "horizontal-rl")
            }
            Self::HorizontalLR => {
                write!(f, "horizontal-lr")
            }
        }
    }
}

#[derive(Debug, FromPyObject)]
struct KrakenBaseline {
    id: String,
    baseline: Vec<Vec<i32>>,
    boundary: Vec<Vec<i32>>,
}

#[derive(Debug, FromPyObject)]
struct KrakenRegion {
    id: String,
    boundary: Vec<Vec<i32>>,
}

pub fn segment_image<P: AsRef<Path>>(
    image_path: P,
    model_path: P,
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
