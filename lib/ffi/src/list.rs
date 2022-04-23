use crate::{call_function, cps, Any, Arc, Boolean, BoxAny, ByteString, Closure};

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
    fn _pen_ffi_list_first_rest(
        stack: &mut cps::AsyncStack<Arc<FirstRest>>,
        continuation: cps::ContinuationFunction<Arc<FirstRest>, Arc<FirstRest>>,
        list: Arc<List>,
    ) -> cps::Result;

    fn _pen_ffi_any_to_string(
        stack: &mut cps::AsyncStack<ByteString>,
        continuation: cps::ContinuationFunction<ByteString, ByteString>,
        value: BoxAny,
    ) -> cps::Result;
}

impl List {
    pub async fn iterate(this: Arc<Self>, mut callback: impl FnMut(Any)) {
        Self::try_iterate(this, |element| -> Result<(), ()> {
            callback(element);

            Ok(())
        })
        .await
        .unwrap();
    }

    pub async fn try_iterate<E>(
        mut list: Arc<Self>,
        mut callback: impl FnMut(Any) -> Result<(), E>,
    ) -> Result<(), E> {
        loop {
            let first_rest = call_function!(
                fn(Arc<List>) -> Arc<FirstRest>,
                _pen_ffi_list_first_rest,
                list.clone(),
            )
            .await;

            if !bool::from(first_rest.ok) {
                break;
            }

            callback(first_rest.first.clone())?;

            list = first_rest.rest.clone();
        }

        Ok(())
    }
}
