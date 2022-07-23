use crate::cps::{AsyncStack, ContinuationFunction, StepFunction};
use core::task::Poll;
use futures::future::poll_fn;

type InitialStepFunction<T> =
    unsafe extern "C" fn(stack: &mut AsyncStack<T>, continuation: ContinuationFunction<T, T>);

const INITIAL_STACK_CAPACITY: usize = 64;

pub async fn from_function<T>(initial_step: InitialStepFunction<T>) -> T {
    let mut trampoline: Option<(StepFunction<(), T>, ContinuationFunction<(), T>)> = None;
    let mut stack = AsyncStack::new(INITIAL_STACK_CAPACITY);

    poll_fn(move |context| {
        stack.run_with_context(context, |stack| {
            if let Some((step, continue_)) = trampoline {
                step(stack, continue_);
            } else {
                unsafe { initial_step(stack, resolve) };
            }
        });

        if let Some(value) = stack.resolved_value() {
            value.into()
        } else {
            trampoline = Some(stack.resume().unwrap());
            Poll::Pending
        }
    })
    .await
}

extern "C" fn resolve<T>(stack: &mut AsyncStack<T>, value: T) {
    stack.resolve(value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Number;

    extern "C" fn foo(
        stack: &mut AsyncStack<Number>,
        continue_: ContinuationFunction<Number, Number>,
    ) {
        continue_(stack, 42.0.into())
    }

    #[tokio::test]
    async fn convert_closure() {
        assert_eq!(from_function(foo).await, 42.0.into());
    }
}
