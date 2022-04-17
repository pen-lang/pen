use crate::{request::Request, response::Response};
use std::error::Error;

#[ffi::any]
#[repr(C)]
#[derive(Clone, Debug)]
struct ResponseResult {
    response: ffi::Arc<Response>,
    error: ffi::ByteString,
}

#[ffi::bindgen]
async fn _pen_http_client_send(request: ffi::Arc<Request>) -> ffi::Arc<ResponseResult> {
    match send_request(request).await {
        Ok(response) => ResponseResult {
            response,
            error: ffi::ByteString::default(),
        }
        .into(),
        Err(error) => ResponseResult {
            response: Default::default(),
            error: error.to_string().into(),
        }
        .into(),
    }
}

async fn send_request(request: ffi::Arc<Request>) -> Result<ffi::Arc<Response>, Box<dyn Error>> {
    let response = hyper::Client::new()
        .request(
            hyper::Request::builder()
                .method(hyper::Method::from_bytes(request.method().as_slice())?)
                .uri(request.uri().as_slice())
                .body(hyper::Body::from(request.body().as_slice().to_vec()))?,
        )
        .await?;

    Ok(Response::new(
        response.status().as_u16() as f64,
        hyper::body::to_bytes(response.into_body()).await?.to_vec(),
    )
    .into())
}
