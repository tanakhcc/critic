//! URLs shared between front- and backend
//!
//! for consistency, all endpoint urls in this module always start with a /

/// URL and filesystem-location to put images into
/// lives under the data-directory in the fs
pub const IMAGE_BASE_LOCATION: &str = "/images";
/// Filesystem Location where transcription data is stored.
pub const TRANSCRIPTION_BASE_LOCATION: &str = "/transcriptions";
/// Base url for static content like files etc.
pub const STATIC_BASE_URL: &str = "/static";
/// The base url for uploading anything
///
/// Please note changes to this value in the README under `Reverse Proxying critic`
pub const UPLOAD_BASE_URL: &str = "/upload";
/// The api endpoint where new manuscript pages should be uploaded to
/// The manuscriptname these pages belong to will be appended after this string (and a /)
pub const PAGE_UPLOAD_API_ENDPOINT: &str = "/v1/page";
