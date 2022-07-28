use futures::{future::FutureExt, pin_mut, Stream};
use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread::available_parallelism,
};
use tokio::{spawn, sync::RwLock, task::yield_now};
use tokio_stream::{StreamExt, StreamMap};

// We hope that keys never conflict until a key integer wraps.
static STREAM_MAP_KEY: AtomicUsize = AtomicUsize::new(0);

#[ffi::bindgen]
async fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
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
    let parallelism = available_parallelism().unwrap().get();
    let stream_map = Arc::new(RwLock::new(Box::pin(StreamMap::with_capacity(
        2 * parallelism,
    ))));
    let cloned_stream_map = stream_map.clone();

    spawn(async move {
        let list = ffi::future::stream::from_list(list);

        pin_mut!(list);

        while let Some(element) = list.next().await {
            cloned_stream_map.write().await.insert(
                STREAM_MAP_KEY.fetch_add(1, Ordering::Relaxed),
                Box::pin(ffi::future::stream::from_list(element.try_into().unwrap())),
            );
        }
    });

    ffi::List::lazy(ffi::future::to_closure(convert_stream_to_list(stream_map)))
}

async fn convert_stream_to_list(
    stream: Arc<RwLock<Pin<Box<impl Stream<Item = (usize, ffi::Any)> + Send + Sync + 'static>>>>,
) -> ffi::Arc<ffi::List> {
    if let Some((_, x)) = stream.write().await.next().await {
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
