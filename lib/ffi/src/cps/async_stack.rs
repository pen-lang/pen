use super::Stack;
use crate::cps;
use std::{
    future::Future,
    intrinsics::transmute,
    ops::{Deref, DerefMut},
    task::Context,
};

type StepFunction<T> = extern "C" fn(
    stack: &mut AsyncStack,
    continuation: extern "C" fn(&mut AsyncStack, T) -> cps::Result,
) -> cps::Result;

type ContinuationFunction<T> = extern "C" fn(&mut AsyncStack, T) -> cps::Result;

#[repr(C)]
#[derive(Debug)]
pub struct AsyncStack {
    stack: Stack,
    context: Option<*mut Context<'static>>,
    suspended: bool,
}

impl AsyncStack {
    pub fn new(capacity: usize) -> Self {
        Self {
            stack: Stack::new(capacity),
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
        step: StepFunction<T>,
        continuation: ContinuationFunction<T>,
        future: impl Future,
    ) {
        self.stack.push(step);
        self.stack.push(continuation);
        self.stack.push(future);

        self.suspended = true;
    }

    pub fn resume<T>(&mut self) -> Option<(StepFunction<T>, ContinuationFunction<T>)> {
        let continuation = self.pop();
        let step = self.pop();

        Some((step, continuation))
    }

    pub fn restore<F: Future>(&mut self) -> Option<F> {
        if self.suspended {
            self.suspended = false;

            Some(self.pop())
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
    fn _test_async_stack_ffi_safety(_: &mut AsyncStack);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::{ready, Ready},
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

    type TestResult = usize;

    extern "C" fn step(
        _: &mut AsyncStack,
        _: extern "C" fn(&mut AsyncStack, TestResult) -> cps::Result,
    ) -> cps::Result {
        cps::Result::new()
    }

    extern "C" fn continuation(_: &mut AsyncStack, _: TestResult) -> cps::Result {
        cps::Result::new()
    }

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

    #[test]
    fn suspend() {
        let waker = create_waker();
        let mut stack = AsyncStack::new(TEST_CAPACITY);
        let mut context = Context::from_waker(&waker);

        stack.set_context(&mut context);
        stack.suspend(step, continuation, ready(42));
    }

    #[test]
    fn suspend_and_resume() {
        let waker = create_waker();
        let mut stack = AsyncStack::new(TEST_CAPACITY);
        let mut context = Context::from_waker(&waker);

        stack.set_context(&mut context);
        stack.suspend(step, continuation, ready(42u64));
        stack.resume::<()>();
        stack.restore::<Ready<u64>>();
    }
}
