use crate::{header_map::HeaderMap, response::Response};
use futures::stream::StreamExt;
use std::error::Error;

#[ffi::bindgen]
async fn _pen_http_client_send(
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

    let keys = ffi::future::stream::from_list(HeaderMap::keys(headers.clone()));

    futures::pin_mut!(keys);

    while let Some(key) = keys.next().await {
        let key = key.to_string().unwrap_or_default();

        builder = builder
            .take()
            .unwrap()
            .header(
                key.as_slice(),
                HeaderMap::get(headers.clone(), key.clone()).as_slice(),
            )
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
        headers = HeaderMap::set(headers, key.as_str(), value.as_bytes());
    }

    Ok(Response::new(
        response.status().as_u16() as f64,
        headers,
        hyper::body::to_bytes(response.into_body()).await?.to_vec(),
    )
    .into())
}
