use async_recursion::async_recursion;
use futures::{future::FutureExt, pin_mut, Stream};
use std::{pin::Pin, sync::Arc};
use tokio::{spawn, sync::RwLock, task::yield_now};
use tokio_stream::{StreamExt, StreamMap};

#[ffi::bindgen]
fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    ffi::future::to_closure(
        spawn(ffi::future::from_closure::<_, ffi::Any>(closure)).map(Result::unwrap),
    )
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
        streams.push(Box::pin(ffi::future::stream::from_list(
            element.try_into().unwrap(),
        )));
    }

    ffi::List::lazy(ffi::future::to_closure(convert_stream_to_list(Arc::new(
        RwLock::new(Box::pin(
            StreamMap::from_iter(streams.into_iter().enumerate()).map(|(_, value)| value),
        )),
    ))))
}

#[async_recursion]
async fn convert_stream_to_list(
    stream: Arc<RwLock<Pin<Box<impl Stream<Item = ffi::Any> + Send + Sync + 'static>>>>,
) -> ffi::Arc<ffi::List> {
    if let Some(x) = stream.write().await.next().await {
        ffi::List::prepend(
            ffi::List::lazy(ffi::future::to_closure(convert_stream_to_list(
                stream.clone(),
            ))),
            x,
        )
    } else {
        ffi::List::new()
    }
}
