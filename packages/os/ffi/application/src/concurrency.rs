use futures::{future::FutureExt, pin_mut, stream::StreamExt};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::thread::available_parallelism;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;
use tokio::{spawn, sync::mpsc::channel, task::yield_now};

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
    let (sender, receiver) = channel(
        2 * available_parallelism()
            .unwrap_or(NonZeroUsize::new(1).unwrap())
            .get(),
    );
    let receiver = Arc::new(RwLock::new(receiver));
    let cloned_receiver = receiver.clone();

    spawn(async move {
        let list = ffi::future::stream::from_list(list);

        pin_mut!(list);

        while let Some(element) = list.next().await {
            let cloned_sender = sender.clone();

            spawn(async move {
                let list = ffi::future::stream::from_list(element.try_into().unwrap());

                pin_mut!(list);

                while let Some(element) = list.next().await {
                    // Ignore send errors.
                    cloned_sender.send(element).await.unwrap_or_default();
                }
            });
        }

        cloned_receiver.write().await.close();
    });

    ffi::List::lazy(ffi::future::to_closure(convert_receiver_to_list(receiver)))
}

async fn convert_receiver_to_list(
    receiver: Arc<RwLock<Receiver<ffi::Any>>>,
) -> ffi::Arc<ffi::List> {
    if let Some(x) = receiver.write().await.recv().await {
        ffi::List::prepend(
            ffi::List::lazy(ffi::future::to_closure(convert_receiver_to_list(
                receiver.clone(),
            ))),
            x,
        )
    } else {
        ffi::List::new()
    }
}
