use super::Stack;
use crate::cps;
use std::{
    future::Future,
    intrinsics::transmute,
    ops::{Deref, DerefMut},
    ptr::null,
    task::Context,
};

type StepFunction<T> = extern "C" fn(
    stack: &mut AsyncStack,
    continuation: extern "C" fn(&mut AsyncStack, T) -> cps::Result,
) -> cps::Result;

type ContinuationFunction<T> = extern "C" fn(&mut AsyncStack, T) -> cps::Result;

#[repr(C)]
pub struct AsyncStack {
    stack: Stack,
    step_function: *const (),
    continuation_function: *const (),
    context: Option<*mut Context<'static>>,
    suspended: bool,
}

impl AsyncStack {
    pub fn new(capacity: usize) -> Self {
        Self {
            stack: Stack::new(capacity),
            step_function: null(),
            continuation_function: null(),
            context: None,
            suspended: false,
        }
    }

    pub fn context(&mut self) -> Option<&mut Context<'_>> {
        self.context.map(|context| unsafe { transmute(context) })
    }

    pub fn set_context(&mut self, context: &mut Context<'_>) {
        self.context = Some(unsafe { transmute(context) });
    }

    pub fn suspend<T>(
        &mut self,
        step_function: StepFunction<T>,
        continuation_function: ContinuationFunction<T>,
        future: impl Future,
    ) {
        self.stack.push(future);
        self.step_function = unsafe { transmute(step_function) };
        self.continuation_function = unsafe { transmute(continuation_function) };
        self.suspended = true;
    }

    pub fn resume<T, F: Future>(
        &mut self,
    ) -> Option<(StepFunction<T>, ContinuationFunction<T>, F)> {
        if self.suspended {
            self.suspended = false;
            Some((
                unsafe { transmute(self.step_function) },
                unsafe { transmute(self.continuation_function) },
                self.stack.pop(),
            ))
        } else {
            None
        }
    }
}

impl Deref for AsyncStack {
    type Target = Stack;

    fn deref(&self) -> &Stack {
        &self.stack
    }
}

impl DerefMut for AsyncStack {
    fn deref_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
}

#[allow(dead_code)]
extern "C" {
    fn _test_async_stack_ffi_safety(_: *mut AsyncStack);
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
