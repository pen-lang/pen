use async_recursion::async_recursion;
use futures::Stream;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::{spawn, sync::RwLock, task::yield_now};
use tokio_stream::{StreamExt, StreamMap};

#[ffi::bindgen]
fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    ffi::future::to_closure(spawn_and_unwrap(ffi::future::from_closure(closure)))
}

async fn spawn_and_unwrap(future: impl Future<Output = ffi::Any> + Send + 'static) -> ffi::Any {
    spawn(future).await.unwrap()
}

#[ffi::bindgen]
async fn _pen_race(list: ffi::Arc<ffi::List>) -> ffi::Arc<ffi::List> {
    let mut streams = vec![];
    let mut first_rest = unsafe { _pen_os_first_rest(list) };

    while first_rest.ok {
        streams.push((
            (),
            Box::pin(convert_list_to_stream(unsafe {
                _pen_os_to_list(ffi::future::from_closure(first_rest.first.clone()).await)
            })),
        ));
        first_rest = unsafe { _pen_os_first_rest(first_rest.rest.clone()) };
    }

    // TODO We should not wait for the first element to be ready.
    convert_stream_to_list(tokio_stream::StreamExt::map(
        StreamMap::from_iter(streams),
        |(_, value)| value,
    ))
    .await
}

// TODO Move this to `pen_ffi::stream::to_stream()`.
fn convert_list_to_stream(list: ffi::Arc<ffi::List>) -> impl Stream<Item = ffi::Any> {
    async_stream::stream! {
        let mut first_rest = unsafe { _pen_os_first_rest(list) };

        while first_rest.ok {
            yield ffi::future::from_closure::<(), ffi::Any>(first_rest.first.clone()).await;
            first_rest = unsafe { _pen_os_first_rest(first_rest.rest.clone()) };
        }
    }
}

// TODO Move this to `pen_ffi::stream::from_stream()`.
async fn convert_stream_to_list(
    stream: impl Stream<Item = ffi::Any> + Send + Sync + 'static,
) -> ffi::Arc<ffi::List> {
    convert_pinned_stream(Arc::new(RwLock::new(Box::pin(stream)))).await
}

#[async_recursion]
async fn convert_pinned_stream(
    stream: Arc<RwLock<Pin<Box<impl Stream<Item = ffi::Any> + Send + Sync + 'static>>>>,
) -> ffi::Arc<ffi::List> {
    if let Some(x) = stream.write().await.next().await {
        let xs = ffi::future::to_closure(convert_pinned_stream(stream.clone()));
        unsafe { _pen_os_create_list(x, xs) }
    } else {
        unsafe { _pen_os_empty_list() }
    }
}

#[ffi::bindgen]
async fn _pen_yield() {
    yield_now().await;
}
