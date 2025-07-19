//! The service actually uploading files (by sending POST requests to the server)

use serde::Deserialize;
use web_sys::FormData;

#[derive(Debug, Default, Clone)]
pub struct FailureReply {
    pub message: String,
}
impl core::fmt::Display for FailureReply {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct BucketDetail {
    pub bucket_id: String,
}
impl core::fmt::Display for BucketDetail {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.bucket_id)
    }
}

/// Transfer files to the api endpoint on the server with a POST request
pub async fn transfer_file(
    files: &Vec<web_sys::File>,
    msname: String,
) -> Result<BucketDetail, FailureReply> {
    let form_data = FormData::new().unwrap();
    for file in files.iter() {
        form_data
            .append_with_blob_and_filename("file", file, file.name().as_str())
            .unwrap();
    }

    match reqwasm::http::Request::post(&format!(
        "{}/{}",
        critic_shared::PAGE_UPLOAD_API_ENDPOINT,
        msname
    ))
    .body(form_data)
    .send()
    .await
    {
        Ok(res) => res
            .json::<BucketDetail>()
            .await
            .map_err(|err| FailureReply {
                message: err.to_string(),
            }),
        Err(err) => Err(FailureReply {
            message: err.to_string(),
        })?,
    }
}
