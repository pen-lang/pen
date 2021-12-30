use super::Stack;
use crate::cps;
use std::{
    future::Future,
    intrinsics::transmute,
    ops::{Deref, DerefMut},
    task::Context,
};

pub type StepFunction<T, S> = unsafe extern "C" fn(
    stack: &mut AsyncStack<S>,
    continuation: ContinuationFunction<T, S>,
) -> cps::Result;

pub type ContinuationFunction<T, S> = unsafe extern "C" fn(&mut AsyncStack<S>, T) -> cps::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AsyncStackAction {
    Suspend,
    Resume,
    Restore,
}

#[repr(C)]
#[derive(Debug)]
pub struct AsyncStack<S> {
    stack: Stack,
    context: Option<*mut Context<'static>>,
    // This field is currently used only for validation in FFI but not by codes
    // generated by the compiler.
    next_action: AsyncStackAction,
    storage: S, // continuation local storage
}

impl<S> AsyncStack<S> {
    pub fn new(capacity: usize, storage: S) -> Self {
        Self {
            stack: Stack::new(capacity),
            context: None,
            next_action: AsyncStackAction::Suspend,
            storage,
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
        step: StepFunction<T, S>,
        continuation: ContinuationFunction<T, S>,
        future: impl Future + Unpin,
    ) {
        if self.next_action != AsyncStackAction::Suspend {
            panic!("continuation not suspendable");
        }

        self.next_action = AsyncStackAction::Resume;

        self.stack.push(future);
        self.stack.push(step);
        self.stack.push(continuation);
    }

    pub fn resume<T>(&mut self) -> (StepFunction<T, S>, ContinuationFunction<T, S>) {
        if self.next_action != AsyncStackAction::Resume {
            panic!("continuation not resumable");
        }

        self.next_action = AsyncStackAction::Restore;

        let continuation = self.pop();
        let step = self.pop();

        (step, continuation)
    }

    pub fn restore<F: Future + Unpin>(&mut self) -> Option<F> {
        if self.next_action != AsyncStackAction::Restore {
            panic!("continuation not restorable");
        }

        self.next_action = AsyncStackAction::Suspend;

        Some(self.pop())
    }

    pub fn storage(&self) -> &S {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut S {
        &mut self.storage
    }
}

impl<S> Deref for AsyncStack<S> {
    type Target = Stack;

    fn deref(&self) -> &Stack {
        &self.stack
    }
}

impl<S> DerefMut for AsyncStack<S> {
    fn deref_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
}

#[allow(dead_code)]
extern "C" {
    fn _test_async_stack_ffi_safety(_: &mut AsyncStack<()>);
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
        _: &mut AsyncStack<()>,
        _: ContinuationFunction<TestResult, ()>,
    ) -> cps::Result {
        cps::Result::new()
    }

    unsafe extern "C" fn continuation(_: &mut AsyncStack<()>, _: TestResult) -> cps::Result {
        cps::Result::new()
    }

    #[test]
    fn push_f64() {
        let mut stack = AsyncStack::new(TEST_CAPACITY, ());

        stack.push(42.0f64);

        assert_eq!(stack.pop::<f64>(), 42.0);
    }

    #[test]
    fn wake() {
        let waker = create_waker();
        let mut stack = AsyncStack::new(TEST_CAPACITY, ());
        let mut context = Context::from_waker(&waker);

        stack.set_context(&mut context);
        stack.context().unwrap().waker().wake_by_ref();
    }

    #[test]
    fn suspend() {
        let waker = create_waker();
        let mut stack = AsyncStack::new(TEST_CAPACITY, ());
        let mut context = Context::from_waker(&waker);

        stack.set_context(&mut context);
        stack.suspend(step, continuation, ready(42));
    }

    #[tokio::test]
    async fn suspend_and_resume() {
        let waker = create_waker();
        let mut stack = AsyncStack::new(TEST_CAPACITY, ());
        let mut context = Context::from_waker(&waker);

        type TestFuture = Ready<()>;

        let future: TestFuture = ready(());

        stack.set_context(&mut context);
        stack.suspend(step, continuation, future);
        stack.resume::<()>();
        stack.restore::<TestFuture>().unwrap().await;
    }

    #[tokio::test]
    #[should_panic]
    async fn panic_on_restore_before_resume() {
        let waker = create_waker();
        let mut stack = AsyncStack::new(TEST_CAPACITY, ());
        let mut context = Context::from_waker(&waker);

        type TestFuture = Ready<()>;

        let future: TestFuture = ready(());

        stack.set_context(&mut context);
        stack.suspend(step, continuation, future);
        stack.restore::<TestFuture>().unwrap().await;
    }
}
