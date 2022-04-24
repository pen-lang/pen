use crate::{call_function, future, import, Any, Arc, Boolean, Closure, List};
use futures::Stream;

pub fn from_list(list: Arc<List>) -> impl Stream<Item = Any> {
    async_stream::stream! {
        let mut first_rest = get_first_rest(list).await;

        while first_rest.ok.into() {
            yield future::from_closure::<(), Any>(first_rest.first.clone()).await;
            first_rest =  get_first_rest(first_rest.rest.clone()).await;
        }
    }
}

async fn get_first_rest(list: Arc<List>) -> Arc<FirstRest> {
    call_function!(
        fn(Arc<List>) -> Arc<FirstRest>,
        _pen_ffi_list_first_rest,
        list.clone(),
    )
    .await
}

#[repr(C)]
struct FirstRest {
    ok: Boolean,
    first: Arc<Closure>,
    rest: Arc<List>,
}

extern "C" {
    import!(
        _pen_ffi_list_first_rest,
        fn(list: Arc<List>) -> Arc<FirstRest>
    );
}
