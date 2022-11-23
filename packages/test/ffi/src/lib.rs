#![no_std]

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use crossbeam::atomic::AtomicCell;

extern crate alloc;

#[repr(C)]
struct State(ffi::Arc<ffi::Any>);

#[ffi::any]
#[derive(Clone)]
struct StateInner(Arc<StateInnerInner>);

struct StateInnerInner {
    cell: AtomicCell<ffi::Any>,
    frozen: AtomicBool,
}

#[ffi::bindgen]
fn _pen_test_state_new() -> State {
    State(ffi::Arc::new(
        StateInner(
            StateInnerInner {
                cell: AtomicCell::new(default_value()),
                frozen: false.into(),
            }
            .into(),
        )
        .into(),
    ))
}

#[ffi::bindgen]
fn _pen_test_state_freeze(state: State) {
    let inner: &StateInner = state.0.as_ref().try_into().unwrap();
    inner.0.frozen.store(true, Ordering::SeqCst);
}

#[ffi::bindgen]
fn _pen_test_state_get(state: State) -> ffi::Any {
    let inner: &StateInner = state.0.as_ref().try_into().unwrap();
    let value = inner.0.cell.swap(default_value());

    inner.0.cell.swap(value.clone());
    value
}

#[ffi::bindgen]
fn _pen_test_state_set(state: State, value: ffi::Any) {
    let inner: &StateInner = state.0.as_ref().try_into().unwrap();
    inner.0.cell.swap(value);
}

fn default_value() -> ffi::Any {
    ffi::None::new().into()
}
