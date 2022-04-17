use crate::{request::Request, response::Response};
use core::str;
use std::error::Error;

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
                Ok::<_, hyper::Error>(hyper::service::service_fn(
                    move |request: hyper::Request<hyper::Body>| {
                        let callback = callback.clone();

                        async move {
                            let callback = callback.clone();
                            let body = hyper::body::to_bytes(request.into_body()).await?;
                            let raw = ffi::call!(
                                fn(ffi::Arc<Request>) -> ffi::Arc<Response>,
                                callback,
                                Request::new(body.to_vec()).into()
                            )
                            .await;

                            Ok::<_, hyper::Error>(
                                if let Ok(status) =
                                    hyper::StatusCode::from_u16(f64::from(raw.status()) as u16)
                                {
                                    let mut response = hyper::Response::new(hyper::Body::from(
                                        raw.body().as_slice().to_vec(),
                                    ));
                                    *response.status_mut() = status;
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
