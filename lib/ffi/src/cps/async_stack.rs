use super::{async_stack_action::AsyncStackAction, CpsError, Stack};
use crate::cps;
use alloc::{vec, vec::Vec};
use core::{
    future::Future,
    intrinsics::transmute,
    ops::{Deref, DerefMut},
    task::Context,
};

pub type StepFunction<T, S = ()> =
    fn(stack: &mut AsyncStack<S>, continuation: ContinuationFunction<T, S>) -> cps::Result;

pub type ContinuationFunction<T, S = ()> = extern "C" fn(&mut AsyncStack<S>, T) -> cps::Result;

pub type Trampoline<T, S> = (StepFunction<T, S>, ContinuationFunction<T, S>);

// Something like AsyncStackManager that creates async stacks with proper
// lifetime every time when it's given contexts can be implemented potentially.
// The reason we set those contexts unsafely with the `run_with_context` method
// in the current implementation is to keep struct type compatible between
// Stack and AsyncStack at the ABI level.
#[repr(C)]
#[derive(Debug)]
pub struct AsyncStack<S = ()> {
    stack: Stack,
    context: Option<*mut Context<'static>>,
    // This field is currently used only for validation in FFI but not by codes
    // generated by the compiler.
    next_actions: Vec<AsyncStackAction>,
    resolved_value: Option<S>,
}

impl<S> AsyncStack<S> {
    pub fn new(capacity: usize) -> Self {
        Self {
            stack: Stack::new(capacity),
            context: None,
            next_actions: vec![AsyncStackAction::Suspend],
            resolved_value: None,
        }
    }

    pub fn context(&mut self) -> Option<&mut Context<'_>> {
        self.context
            .map(|context| unsafe { &mut *transmute::<_, *mut Context<'_>>(context) })
    }

    pub fn run_with_context<T>(
        &mut self,
        context: &mut Context<'_>,
        callback: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.context = Some(unsafe { transmute(context) });

        let value = callback(self);

        self.context = None;

        value
    }

    pub fn suspend<T>(
        &mut self,
        step: StepFunction<T, S>,
        continuation: ContinuationFunction<T, S>,
        future: impl Future + Unpin,
    ) -> Result<(), CpsError> {
        self.validate_action(AsyncStackAction::Suspend)?;
        self.push_next_actions(&[
            AsyncStackAction::Resume,
            AsyncStackAction::Restore,
            AsyncStackAction::Suspend,
        ]);

        self.stack.push(future);
        self.stack.push(step);
        self.stack.push(continuation);

        Ok(())
    }

    // Trampoilne a continuation function call to call it from the near bottom
    // of stack clearing the current stack frames.
    // Without this due to the lack of tail call elimination in Rust, machine
    // stacks can grow arbitrarily deep.
    pub fn trampoline<T>(
        &mut self,
        continuation: ContinuationFunction<T, S>,
        value: T,
    ) -> Result<(), CpsError> {
        self.validate_action(AsyncStackAction::Suspend)?;
        self.push_next_actions(&[AsyncStackAction::Resume, AsyncStackAction::Suspend]);

        fn step<T, S>(
            stack: &mut AsyncStack<S>,
            continue_: ContinuationFunction<T, S>,
        ) -> cps::Result {
            let value = stack.pop::<T>();

            continue_(stack, value)
        }

        let step: StepFunction<T, S> = step;

        self.stack.push(value);
        self.stack.push(step);
        self.stack.push(continuation);

        Ok(())
    }

    pub fn resume<T>(&mut self) -> Result<Trampoline<T, S>, CpsError> {
        self.validate_action(AsyncStackAction::Resume)?;

        let continuation = self.pop();
        let step = self.pop();

        Ok((step, continuation))
    }

    pub fn restore<F: Future + Unpin>(&mut self) -> Result<F, CpsError> {
        self.validate_action(AsyncStackAction::Restore)?;

        Ok(self.pop())
    }

    pub fn resolved_value(&mut self) -> Option<S> {
        self.resolved_value.take()
    }

    pub fn resolve(&mut self, value: S) {
        self.resolved_value = Some(value);
    }

    fn validate_action(&mut self, current_action: AsyncStackAction) -> Result<(), CpsError> {
        let next_action = self.next_actions.pop();

        if next_action != Some(current_action) {
            return Err(CpsError::UnexpectedAsyncStackAction(next_action));
        }

        Ok(())
    }

    fn push_next_actions(&mut self, next_actions: &[AsyncStackAction]) {
        self.next_actions.extend(next_actions.iter().rev().copied());
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

// We can mark async stacks Send + Send because:
//
// - Stack should implement Send + Sync. Currently, we don't as we don't need
//   to.
// - Option<*mut Context> is cleared to None on every non-preemptive run.
unsafe impl<S: Send> Send for AsyncStack<S> {}

unsafe impl<S: Sync> Sync for AsyncStack<S> {}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{
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

    fn step(_: &mut AsyncStack, _: ContinuationFunction<TestResult, ()>) -> cps::Result {
        cps::Result::new()
    }

    extern "C" fn continuation(_: &mut AsyncStack, _: TestResult) -> cps::Result {
        cps::Result::new()
    }

    #[allow(dead_code)]
    extern "C" {
        fn _test_async_stack_ffi_safety(_: &mut AsyncStack);
    }

    #[test]
    fn push_f64() {
        let mut stack = AsyncStack::<()>::new(TEST_CAPACITY);

        stack.push(42.0f64);

        assert_eq!(stack.pop::<f64>(), 42.0);
    }

    #[test]
    fn wake() {
        let waker = create_waker();
        let mut stack = AsyncStack::<()>::new(TEST_CAPACITY);
        let mut context = Context::from_waker(&waker);

        stack.run_with_context(&mut context, |stack| {
            stack.context().unwrap().waker().wake_by_ref()
        });
    }

    #[test]
    fn suspend() {
        let mut stack = AsyncStack::new(TEST_CAPACITY);

        stack.suspend(step, continuation, ready(42)).unwrap();
    }

    #[tokio::test]
    async fn suspend_and_resume() {
        let mut stack = AsyncStack::new(TEST_CAPACITY);

        type TestFuture = Ready<usize>;

        let future: TestFuture = ready(42);

        stack.suspend(step, continuation, future).unwrap();
        stack.resume::<()>().unwrap();
        assert_eq!(stack.restore::<TestFuture>().unwrap().await, 42);
    }

    #[tokio::test]
    async fn fail_to_restore_before_resume() {
        let mut stack = AsyncStack::new(TEST_CAPACITY);

        type TestFuture = Ready<()>;

        let future: TestFuture = ready(());

        stack.suspend(step, continuation, future).unwrap();
        assert_eq!(
            stack.restore::<TestFuture>().unwrap_err(),
            CpsError::UnexpectedAsyncStackAction(Some(AsyncStackAction::Resume))
        );
    }
}
