//! The service actually uploading files (by sending POST requests to the server)

use critic_shared::FileTransferOkResponse;
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

/// Transfer files to the api endpoint on the server with a POST request
pub async fn transfer_file(
    files: &[web_sys::File],
    msname: String,
) -> Result<FileTransferOkResponse, FailureReply> {
    let form_data = FormData::new().unwrap();
    for file in files.iter() {
        form_data
            .append_with_blob_and_filename("file", file, file.name().as_str())
            .unwrap();
    }

    match reqwasm::http::Request::post(&format!(
        "{}{}/{}",
        critic_shared::urls::UPLOAD_BASE_URL,
        critic_shared::urls::PAGE_UPLOAD_API_ENDPOINT,
        msname
    ))
    .body(form_data)
    .send()
    .await
    {
        // TODO: we can be smarter then this and actually get the error message the server gave us
        Ok(res) => res
            .json::<FileTransferOkResponse>()
            .await
            .map_err(|err| FailureReply {
                message: err.to_string(),
            }),
        Err(err) => Err(FailureReply {
            message: err.to_string(),
        })?,
    }
}
