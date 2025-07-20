//! The service actually uploading files (by sending POST requests to the server)

use critic_shared::{FileTransferResponse, MAX_BODY_SIZE};
use web_sys::FormData;

pub async fn transfer_batch(files: &[web_sys::File], msname: &str) -> FileTransferResponse {
    let form_data = FormData::new().unwrap();
    for file in files.iter() {
        form_data
            .append_with_blob_and_filename("file", file, file.name().as_str())
            .unwrap();
    }

    let mut this_batch_response = FileTransferResponse::new();

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
        Ok(res) => match res.json::<FileTransferResponse>().await {
            Ok(x) => {
                this_batch_response = x;
            }
            Err(e) => {
                this_batch_response.push_err_batch(
                    format!("There was a problem deserializing response: {e}."),
                    files.len(),
                );
            }
        },
        Err(e) => {
            this_batch_response.push_err_batch(
                format!("There was a problem sending the POST request: {e}."),
                files.len(),
            );
        }
    };
    this_batch_response
}

/// Transfer files to the api endpoint on the server with a POST request
pub async fn transfer_files(files: &[web_sys::File], msname: &str) -> FileTransferResponse {
    let mut response = FileTransferResponse::new();
    // loop; take as many files as possible until the upload limit is reached
    // send a batch, update the response with the results
    let mut batch_start = 0;
    let mut batch_end = 0;
    let mut file_iter = files.iter();
    let mut current_batch_size = 0_f64;
    while let Some(file) = file_iter.next() {
        if file.size() + current_batch_size < MAX_BODY_SIZE as f64 {
            current_batch_size += file.size();
            batch_end += 1;
        // `file` would make this batch to large. send the last one
        } else {
            // send this batch
            response.extend(
                transfer_batch(&files[batch_start..batch_end], msname)
                    .await
                    .err
                    .into_iter(),
            );
            // start a new batch - this starts with (and contains) the file we are currently on
            batch_start = batch_end;
            batch_end = batch_start + 1;
            current_batch_size = file.size();

            // file is individually to large - error out for this file and skip it
            if file.size() > MAX_BODY_SIZE as f64 {
                response.push_err("File is to large.".to_string());
                // the batch now contains no files
                batch_start += 1;
                current_batch_size = 0_f64;
            };
        };
    }
    // send the final batch
    response.extend(
        transfer_batch(&files[batch_start..batch_end], msname)
            .await
            .err
            .into_iter(),
    );
    // and return the responses
    response
}
