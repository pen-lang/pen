use super::Stack;
use crate::cps;
use std::{
    future::Future,
    intrinsics::transmute,
    ops::{Deref, DerefMut},
    task::Context,
};

pub type StepFunction<T> = unsafe extern "C" fn(
    stack: &mut AsyncStack,
    continuation: ContinuationFunction<T>,
) -> cps::Result;

pub type ContinuationFunction<T> = unsafe extern "C" fn(&mut AsyncStack, T) -> cps::Result;

#[repr(C)]
#[derive(Debug)]
pub struct AsyncStack {
    stack: Stack,
    context: Option<*mut Context<'static>>,
    // TODO Replace with an enum representing async stack states.
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
        #[allow(clippy::transmute_ptr_to_ref)]
        self.context.map(|context| unsafe { transmute(context) })
    }

    pub fn set_context(&mut self, context: &mut Context<'_>) {
        self.context = Some(unsafe { transmute(context) });
    }

    pub fn suspend<T>(
        &mut self,
        step: StepFunction<T>,
        continuation: ContinuationFunction<T>,
        future: impl Future + Unpin,
    ) {
        self.stack.push(future);
        self.stack.push(step);
        self.stack.push(continuation);

        self.suspended = true;
    }

    pub fn resume<T>(&mut self) -> (StepFunction<T>, ContinuationFunction<T>) {
        let continuation = self.pop();
        let step = self.pop();

        (step, continuation)
    }

    pub fn restore<F: Future + Unpin>(&mut self) -> Option<F> {
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

    unsafe extern "C" fn step(
        _: &mut AsyncStack,
        _: ContinuationFunction<TestResult>,
    ) -> cps::Result {
        cps::Result::new()
    }

    unsafe extern "C" fn continuation(_: &mut AsyncStack, _: TestResult) -> cps::Result {
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

    #[tokio::test]
    async fn suspend_and_resume() {
        let waker = create_waker();
        let mut stack = AsyncStack::new(TEST_CAPACITY);
        let mut context = Context::from_waker(&waker);

        type TestFuture = Ready<()>;

        let future: TestFuture = ready(());

        stack.set_context(&mut context);
        stack.suspend(step, continuation, future);
        stack.resume::<()>();
        stack.restore::<TestFuture>().unwrap().await;
    }
}
