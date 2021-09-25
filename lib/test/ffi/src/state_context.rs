use ffi::AnyLike;
use std::sync::{Arc, RwLock};

#[repr(C)]
#[derive(Clone)]
pub struct StateContext {
    inner: ffi::Any,
}

#[repr(C)]
#[derive(Clone)]
pub struct StateContextInner {
    pub state: Arc<RwLock<ffi::Any>>,
}

ffi::type_information!(state_context, crate::state_context::StateContextInner);

impl StateContext {
    pub fn new() -> Self {
        Self {
            inner: StateContextInner {
                state: RwLock::new(ffi::None::new().into_any()).into(),
            }
            .into_any(),
        }
    }

    pub fn get(&self) -> ffi::Any {
        self.inner().state.read().unwrap().clone()
    }

    pub fn set(&self, state: ffi::Any) {
        *self.inner().state.write().unwrap() = state;
    }

    fn inner(&self) -> &StateContextInner {
        StateContextInner::as_inner(&self.inner).unwrap()
    }
}

#[no_mangle]
fn _pen_test_state_context_create() -> ffi::Arc<StateContext> {
    StateContext::new().into()
}

#[no_mangle]
fn _pen_test_state_context_get(context: ffi::Arc<StateContext>) -> ffi::Any {
    context.get()
}

#[no_mangle]
fn _pen_test_state_context_set(context: ffi::Arc<StateContext>, state: ffi::Any) {
    context.set(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_state() {
        assert_eq!(
            ffi::None::from_any(_pen_test_state_context_get(_pen_test_state_context_create())),
            Some(ffi::None::new())
        );
    }

    #[test]
    fn set_state() {
        let context = _pen_test_state_context_create();

        _pen_test_state_context_set(context.clone(), ffi::Number::new(42.0).into_any());

        assert_eq!(
            ffi::Number::from_any(_pen_test_state_context_get(context)),
            Some(ffi::Number::new(42.0))
        );
    }
}
