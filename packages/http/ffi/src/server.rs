use crate::{header_map::HeaderMap, response::Response};
use core::str;
use futures::stream::StreamExt;
use hyper::header::{HeaderName, HeaderValue};
use std::error::Error;

type BoxError = Box<dyn Error + Send + Sync + 'static>;

#[ffi::bindgen]
async fn _pen_http_server_serve(
    address: ffi::ByteString,
    callback: ffi::Closure,
) -> Result<(), Box<dyn Error>> {
    hyper::Server::try_bind(&str::from_utf8(address.as_slice())?.parse()?)?
        .serve(hyper::service::make_service_fn(|_| {
            let callback = callback.clone();

            async {
                Ok::<_, BoxError>(hyper::service::service_fn(
                    move |request: hyper::Request<hyper::Body>| {
                        let callback = callback.clone();

                        async {
                            let method = request.method().to_string();
                            let uri = request.uri().to_string();
                            let mut headers = HeaderMap::new();

                            for (key, value) in request.headers() {
                                headers = headers.set(key.as_str(), value.as_bytes());
                            }

                            let body = hyper::body::to_bytes(request.into_body()).await?;

                            let response = ffi::call!(
                                fn(
                                    ffi::ByteString,
                                    ffi::ByteString,
                                    HeaderMap,
                                    ffi::ByteString,
                                ) -> Response,
                                callback,
                                method.into(),
                                uri.into(),
                                headers,
                                body.to_vec().into()
                            )
                            .await;

                            let mut builder = Some(hyper::Response::builder().status(
                                hyper::StatusCode::from_u16(f64::from(response.status()) as u16)?,
                            ));

                            let keys = ffi::future::stream::from_list(response.headers().keys());

                            futures::pin_mut!(keys);

                            while let Some(key) = keys.next().await {
                                let key = ffi::ByteString::try_from(key).unwrap_or_default();

                                builder = builder
                                    .take()
                                    .unwrap()
                                    .header(
                                        HeaderName::from_bytes(key.as_slice())?,
                                        HeaderValue::from_bytes(
                                            response.headers().get(key.clone()).as_slice(),
                                        )?,
                                    )
                                    .into();
                            }

                            Ok::<_, BoxError>(
                                builder
                                    .take()
                                    .unwrap()
                                    .body(hyper::Body::from(response.body().as_slice().to_vec()))?,
                            )
                        }
                    },
                ))
            }
        }))
        .await?;

    Ok(())
}
