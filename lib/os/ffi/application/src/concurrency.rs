use futures::{stream, Stream};
use std::future::Future;
use tokio::spawn;
use tokio_stream::{StreamExt, StreamMap};

#[repr(C)]
struct FirstRest {
    ok: bool,
    first: ffi::Any,
    rest: ffi::Arc<ffi::List>,
}

extern "C" {
    fn _pen_os_first_rest(list: ffi::Arc<ffi::List>) -> ffi::Arc<FirstRest>;
    fn _pen_os_to_list(list: ffi::Any) -> ffi::Arc<ffi::List>;
}

#[ffi::bindgen]
fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    ffi::future::to_closure(spawn_and_unwrap(ffi::future::from_closure(closure)))
}

async fn spawn_and_unwrap(future: impl Future<Output = ffi::Any> + Send + 'static) -> ffi::Any {
    spawn(future).await.unwrap()
}

#[ffi::bindgen]
async fn _pen_join(list: ffi::Arc<ffi::List>) -> ffi::Arc<ffi::List> {
    let mut streams = vec![];
    let mut first_rest = unsafe { _pen_os_first_rest(list) };

    while first_rest.ok {
        streams.push((
            (),
            convert_list_to_stream(unsafe { _pen_os_to_list(first_rest.first.clone()) }),
        ));
        first_rest = unsafe { _pen_os_first_rest(first_rest.rest.clone()) };
    }

    convert_stream_to_list(StreamMap::from_iter(streams).map(|(_, value)| value))
}

// TODO Move this to `pen_ffi::stream::to_stream()`.
fn convert_list_to_stream(_list: ffi::Arc<ffi::List>) -> impl Stream<Item = ffi::Any> {
    todo!();

    #[allow(unreachable_code)]
    stream::iter(vec![])
}

// TODO Move this to `pen_ffi::stream::from_stream()`.
fn convert_stream_to_list(_stream: impl Stream<Item = ffi::Any>) -> ffi::Arc<ffi::List> {
    todo!();
}
