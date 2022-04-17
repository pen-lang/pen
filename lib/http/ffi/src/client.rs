use crate::{request::Request, response::Response};
use std::error::Error;

#[ffi::any]
#[repr(C)]
#[derive(Clone, Debug)]
struct FfiResponse {
    response: ffi::Arc<Response>,
    error: ffi::ByteString,
}

#[ffi::bindgen]
async fn _pen_http_client_send(request: ffi::Arc<Request>) -> ffi::Arc<FfiResponse> {
    match send(request).await {
        Ok(response) => FfiResponse {
            response,
            error: ffi::ByteString::default(),
        }
        .into(),
        Err(error) => FfiResponse {
            response: Default::default(),
            error: error.to_string().into(),
        }
        .into(),
    }
}

async fn send(_request: ffi::Arc<Request>) -> Result<ffi::Arc<Response>, Box<dyn Error>> {
    todo!()
}
