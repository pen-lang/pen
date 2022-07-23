use async_recursion::async_recursion;
use futures::{pin_mut, Stream};
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
async fn _pen_yield() {
    yield_now().await;
}

#[ffi::bindgen]
async fn _pen_race(list: ffi::Arc<ffi::List>) -> ffi::Arc<ffi::List> {
    let list = ffi::future::stream::from_list(list);

    pin_mut!(list);

    let mut streams = vec![];

    while let Some(element) = list.next().await {
        streams.push((
            (),
            Box::pin(ffi::future::stream::from_list(element.try_into().unwrap())),
        ));
    }

    convert_stream_to_list(Arc::new(RwLock::new(Box::pin(
        StreamMap::from_iter(streams).map(|(_, value)| value),
    ))))
    .await
}

// TODO Use List::lazy(). We should not wait for the first element to be ready.
#[async_recursion]
async fn convert_stream_to_list(
    stream: Arc<RwLock<Pin<Box<impl Stream<Item = ffi::Any> + Send + Sync + 'static>>>>,
) -> ffi::Arc<ffi::List> {
    if let Some(x) = stream.write().await.next().await {
        ffi::List::prepend(convert_stream_to_list(stream.clone()).await, x)
    } else {
        ffi::List::new()
    }
}
