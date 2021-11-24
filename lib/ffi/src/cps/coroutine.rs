use super::AsyncStack;
use crate::cps;
use std::{
    future::Future,
    intrinsics::transmute,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

type StepFunction<T> = extern "C" fn(
    stack: &mut AsyncStack,
    continuation: extern "C" fn(&mut AsyncStack, T) -> cps::Result,
) -> cps::Result;

type ContinuationFunction<T> = extern "C" fn(&mut AsyncStack, T) -> cps::Result;

#[repr(C)]
pub struct Coroutine<T> {
    stack: AsyncStack,
    context: Option<*mut Context<'static>>,
    step_function: *const (),
    continuation_function: *const (),
    _result: PhantomData<T>,
}

extern "C" fn push_result<T>(stack: &mut cps::AsyncStack, result: T) -> cps::Result {
    stack.push(result);

    cps::Result::new()
}

impl<T> Coroutine<T> {
    pub fn new(
        capacity: usize,
        step_function: fn(
            stack: &mut AsyncStack,
            continuation: extern "C" fn(&mut AsyncStack, T) -> cps::Result,
        ) -> cps::Result,
    ) -> Self {
        Self {
            stack: AsyncStack::new(capacity),
            context: None,
            step_function: step_function as *const (),
            continuation_function: push_result::<T> as *const (),
            _result: Default::default(),
        }
    }
}

impl<T: Unpin> Future for Coroutine<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<T> {
        let step_function = unsafe { transmute::<_, StepFunction<()>>(self.step_function) };
        let continuation_function =
            unsafe { transmute::<_, ContinuationFunction<()>>(self.continuation_function) };

        self.stack.set_context(context);
        step_function(&mut self.stack, continuation_function);

        // TODO Check if a result is ready.
        Poll::Pending
    }
}
