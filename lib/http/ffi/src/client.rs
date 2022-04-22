use crate::{header_map::HeaderMap, response::Response};
use std::error::Error;

#[ffi::any]
#[repr(C)]
#[derive(Clone, Debug)]
struct ResponseResult {
    response: ffi::Arc<Response>,
    error: ffi::ByteString,
}

#[ffi::bindgen]
async fn _pen_http_client_send(
    method: ffi::ByteString,
    uri: ffi::ByteString,
    headers: ffi::Arc<HeaderMap>,
    body: ffi::ByteString,
) -> ffi::Arc<ResponseResult> {
    match send_request(method, uri, headers, body).await {
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

async fn send_request(
    method: ffi::ByteString,
    uri: ffi::ByteString,
    headers: ffi::Arc<HeaderMap>,
    body: ffi::ByteString,
) -> Result<ffi::Arc<Response>, Box<dyn Error>> {
    let mut builder = Some(
        hyper::Request::builder()
            .method(hyper::Method::from_bytes(method.as_slice())?)
            .uri(uri.as_slice()),
    );

    HeaderMap::iterate(&headers, |key, value| {
        builder = Some(
            builder
                .take()
                .unwrap()
                .header(key.as_slice(), value.as_slice()),
        );
    });

    let response = hyper::Client::new()
        .request(
            builder
                .unwrap()
                .body(hyper::Body::from(body.as_slice().to_vec()))?,
        )
        .await?;

    Ok(Response::new(
        response.status().as_u16() as f64,
        hyper::body::to_bytes(response.into_body()).await?.to_vec(),
    )
    .into())
}
