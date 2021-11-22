use super::Stack;
use std::{
    future::Future,
    ops::{Deref, DerefMut},
    task::Context,
};

#[repr(C)]
pub struct AsyncStack<'a, 'b> {
    stack: Stack,
    context: Option<&'a mut Context<'b>>,
    suspended: bool,
}

impl<'a, 'b> AsyncStack<'a, 'b> {
    pub fn new(capacity: usize) -> Self {
        Self {
            stack: Stack::new(capacity),
            context: None,
            suspended: false,
        }
    }

    pub fn context(&mut self) -> Option<&mut Context<'b>> {
        self.context.as_deref_mut()
    }

    pub fn set_context(&mut self, context: &'a mut Context<'b>) {
        self.context = Some(context);
    }

    pub fn suspend(&mut self, future: impl Future) {
        self.stack.push(future);
        self.suspended = true;
    }

    pub fn resume<F: Future>(&mut self) -> Option<F> {
        if self.suspended {
            self.suspended = false;
            Some(self.stack.pop())
        } else {
            None
        }
    }
}

impl<'a, 'b> Deref for AsyncStack<'a, 'b> {
    type Target = Stack;

    fn deref(&self) -> &Stack {
        &self.stack
    }
}

impl<'a, 'b> DerefMut for AsyncStack<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
}

#[allow(dead_code)]
extern "C" {
    fn _test_async_stack_ffi_safety(_: *mut AsyncStack<'_, '_>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ptr::null,
        task::{RawWaker, RawWakerVTable, Waker},
    };

    const TEST_CAPACITY: usize = 1;
    const RAW_WAKER_DATA: () = ();
    const RAW_WAKER_V_TABLE: RawWakerVTable =
        RawWakerVTable::new(clone_waker, do_nothing, do_nothing, do_nothing);

    fn create_waker() -> Waker {
        unsafe { Waker::from_raw(RawWaker::new(&RAW_WAKER_DATA, &RAW_WAKER_V_TABLE)) }
    }

    fn clone_waker(_: *const ()) -> RawWaker {
        RawWaker::new(null(), &RAW_WAKER_V_TABLE)
    }

    fn do_nothing(_: *const ()) {}

    #[test]
    fn push_f64() {
        let mut stack = AsyncStack::new(TEST_CAPACITY);

        stack.push(42.0f64);

        assert_eq!(stack.pop::<f64>(), 42.0);
    }

    #[test]
    fn wake() {
        let waker = create_waker();
        let mut stack = AsyncStack::new(TEST_CAPACITY);
        let mut context = Context::from_waker(&waker);

        stack.set_context(&mut context);
        stack.context().unwrap().waker().wake_by_ref();
    }
}
