use core::str;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response,
};
use std::error::Error;

type Callback = extern "C" fn(ffi::None) -> ffi::None;

#[ffi::bindgen]
fn _pen_http_server_serve(address: ffi::ByteString, callback: Callback) -> ffi::ByteString {
    match serve(address, callback) {
        Ok(_) => ffi::ByteString::default(),
        Err(error) => error.to_string().into(),
    }
}

fn serve(address: ffi::ByteString, callback: Callback) -> Result<(), Box<dyn Error>> {
    hyper::Server::try_bind(&str::from_utf8(address.as_slice())?.parse()?)?.serve(make_service_fn(
        |_| async {
            Ok::<_, hyper::Error>(service_fn(|_| async {
                Ok::<_, hyper::Error>(Response::new(Body::from("Hello World")))
            }))
        },
    ));

    Ok(())
}
