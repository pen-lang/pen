use crate::{Any, Arc, Boolean, Closure, List, future, import};
use futures::Stream;

import!(pen_ffi_list_first_rest, async fn(list: List) -> Arc<FirstRest>);

#[repr(C)]
struct FirstRest {
    ok: Boolean,
    first: Closure,
    rest: List,
}

pub fn from_list(list: List) -> impl Stream<Item = Any> {
    async_stream::stream! {
        let mut first_rest = get_first_rest(list).await;

        while first_rest.ok.into() {
            yield future::from_closure::<(), Any>(first_rest.first.clone()).await;
            first_rest = get_first_rest(first_rest.rest.clone()).await;
        }
    }
}

async fn get_first_rest(list: List) -> Arc<FirstRest> {
    pen_ffi_list_first_rest(list).await
}
