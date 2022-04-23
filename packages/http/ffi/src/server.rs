use crate::{header_map::HeaderMap, response::Response};
use core::str;
use hyper::header::{HeaderName, HeaderValue};
use std::error::Error;

type BoxError = Box<dyn Error + Send + Sync + 'static>;

#[ffi::bindgen]
async fn _pen_http_server_serve(
    address: ffi::ByteString,
    callback: ffi::Arc<ffi::Closure>,
) -> ffi::ByteString {
    match serve(address, callback).await {
        Ok(_) => ffi::ByteString::default(),
        Err(error) => error.to_string().into(),
    }
}

async fn serve(
    address: ffi::ByteString,
    callback: ffi::Arc<ffi::Closure>,
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
                                headers = HeaderMap::set(&headers, key.as_str(), value.as_bytes());
                            }

                            let body = hyper::body::to_bytes(request.into_body()).await?;

                            let raw = ffi::call!(
                                fn(
                                    ffi::ByteString,
                                    ffi::ByteString,
                                    ffi::Arc<HeaderMap>,
                                    ffi::ByteString,
                                ) -> ffi::Arc<Response>,
                                callback,
                                method.into(),
                                uri.into(),
                                HeaderMap::new(),
                                body.to_vec().into()
                            )
                            .await;

                            Ok::<_, BoxError>(
                                if let Ok(status) =
                                    hyper::StatusCode::from_u16(f64::from(raw.status()) as u16)
                                {
                                    let mut response = hyper::Response::new(hyper::Body::from(
                                        raw.body().as_slice().to_vec(),
                                    ));

                                    *response.status_mut() = status;

                                    HeaderMap::try_iterate(
                                        &raw.headers(),
                                        |key, value| -> Result<(), BoxError> {
                                            response.headers_mut().insert(
                                                HeaderName::from_bytes(key.as_slice())?,
                                                HeaderValue::from_bytes(value.as_slice())?,
                                            );

                                            Ok(())
                                        },
                                    )?;

                                    response
                                } else {
                                    let mut response = hyper::Response::new(hyper::Body::from(
                                        "Invalid status code",
                                    ));

                                    *response.status_mut() =
                                        hyper::StatusCode::INTERNAL_SERVER_ERROR;

                                    response
                                },
                            )
                        }
                    },
                ))
            }
        }))
        .await?;

    Ok(())
}
