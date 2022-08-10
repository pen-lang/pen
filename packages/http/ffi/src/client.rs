use crate::{header_map::HeaderMap, response::Response};
use futures::stream::StreamExt;
use std::error::Error;

#[ffi::bindgen]
async fn _pen_http_client_send(
    method: ffi::ByteString,
    uri: ffi::ByteString,
    headers: HeaderMap,
    body: ffi::ByteString,
) -> Result<Response, Box<dyn Error>> {
    let mut builder = Some(
        hyper::Request::builder()
            .method(hyper::Method::from_bytes(method.as_slice())?)
            .uri(uri.as_slice()),
    );

    let keys = ffi::future::stream::from_list(headers.keys());

    futures::pin_mut!(keys);

    while let Some(key) = keys.next().await {
        let key = ffi::ByteString::try_from(key).unwrap_or_default();

        builder = builder
            .take()
            .unwrap()
            .header(key.as_slice(), headers.get(key.clone()).as_slice())
            .into();
    }

    let response = hyper::Client::new()
        .request(
            builder
                .unwrap()
                .body(hyper::Body::from(body.as_slice().to_vec()))?,
        )
        .await?;
    let mut headers = HeaderMap::new();

    for (key, value) in response.headers() {
        headers = headers.set(key.as_str(), value.as_bytes());
    }

    Ok(Response::new(
        response.status().as_u16() as f64,
        headers,
        hyper::body::to_bytes(response.into_body()).await?.to_vec(),
    ))
}
