use futures::{future::FutureExt, pin_mut, stream::StreamExt};
use std::{num::NonZeroUsize, thread::available_parallelism};
use tokio::{
    spawn,
    sync::mpsc::{Receiver, channel},
    task::yield_now,
};

const PARALLELISM_MULTIPLIER: usize = 2;

#[ffi::bindgen]
async fn _pen_spawn(closure: ffi::Closure) -> ffi::Closure {
    ffi::future::to_closure(
        spawn(ffi::future::from_closure::<_, ffi::Any>(closure)).map(Result::unwrap),
    )
}

#[ffi::bindgen]
async fn _pen_yield() {
    yield_now().await;
}

#[ffi::bindgen]
async fn _pen_race(list: ffi::List) -> ffi::List {
    let (sender, receiver) = channel(
        PARALLELISM_MULTIPLIER
            * available_parallelism()
                .unwrap_or(NonZeroUsize::new(1).unwrap())
                .get(),
    );

    spawn(async move {
        let list = ffi::future::stream::from_list(list);

        pin_mut!(list);

        while let Some(element) = list.next().await {
            let cloned_sender = sender.clone();

            spawn(async move {
                let list = ffi::future::stream::from_list(element.try_into().unwrap());

                pin_mut!(list);

                while let Some(element) = list.next().await {
                    cloned_sender.send(element).await.unwrap_or_default();
                }
            });
        }
    });

    ffi::List::lazy(ffi::future::to_closure(convert_receiver_to_list(receiver)))
}

async fn convert_receiver_to_list(mut receiver: Receiver<ffi::Any>) -> ffi::List {
    if let Some(x) = receiver.recv().await {
        ffi::List::prepend(
            ffi::List::lazy(ffi::future::to_closure(convert_receiver_to_list(receiver))),
            x,
        )
    } else {
        ffi::List::new()
    }
}
