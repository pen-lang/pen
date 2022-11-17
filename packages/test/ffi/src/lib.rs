#![no_std]

use alloc::sync::Arc;
use crossbeam::atomic::AtomicCell;

extern crate alloc;

#[repr(C)]
pub struct State(ffi::Arc<ffi::Any>);

#[ffi::any]
#[derive(Clone)]
struct StateInner {
    cell: Arc<AtomicCell<ffi::Any>>,
}

#[ffi::bindgen]
fn _pen_test_state_new() -> State {
    State(ffi::Arc::new(
        StateInner {
            cell: AtomicCell::new(default_value()).into(),
        }
        .into(),
    ))
}

#[ffi::bindgen]
fn _pen_test_state_get(state: State) -> ffi::Any {
    let inner: &StateInner = (&*state.0).try_into().unwrap();
    let value = inner.cell.swap(default_value());

    inner.cell.swap(value.clone());
    value
}

#[ffi::bindgen]
fn _pen_test_state_set(state: State, value: ffi::Any) {
    let inner: &StateInner = (&*state.0).try_into().unwrap();
    inner.cell.swap(value);
}

fn default_value() -> ffi::Any {
    ffi::None::new().into()
}
