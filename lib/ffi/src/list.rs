use crate::{call_function, import, Any, Arc, Boolean, Closure};
use core::future::Future;

#[pen_ffi_macro::any(crate = "crate")]
#[repr(C)]
#[derive(Clone)]
pub struct List {
    node: Arc<Closure>,
}

#[repr(C)]
struct FirstRest {
    ok: Boolean,
    first: Any,
    rest: Arc<List>,
}

extern "C" {
    import!(
        _pen_ffi_list_first_rest,
        fn(list: Arc<List>) -> Arc<FirstRest>
    );
}

impl List {
    pub async fn iterate<F: Future<Output = ()>>(
        mut list: Arc<Self>,
        mut callback: impl FnMut(Any) -> F,
    ) {
        loop {
            let first_rest = List::first_rest(list.clone()).await;

            if !bool::from(first_rest.ok) {
                break;
            }

            callback(first_rest.first.clone()).await;

            list = first_rest.rest.clone();
        }
    }

    pub async fn try_iterate<E, F: Future<Output = Result<(), E>>>(
        mut list: Arc<Self>,
        mut callback: impl FnMut(Any) -> F,
    ) -> Result<(), E> {
        loop {
            let first_rest = List::first_rest(list.clone()).await;

            if !bool::from(first_rest.ok) {
                break;
            }

            callback(first_rest.first.clone()).await?;

            list = first_rest.rest.clone();
        }

        Ok(())
    }

    async fn first_rest(list: Arc<Self>) -> Arc<FirstRest> {
        call_function!(
            fn(Arc<List>) -> Arc<FirstRest>,
            _pen_ffi_list_first_rest,
            list.clone(),
        )
        .await
    }
}
