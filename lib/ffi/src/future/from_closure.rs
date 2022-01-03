use crate::{
    cps::{self, AsyncStack, ContinuationFunction, StepFunction},
    Arc, Closure,
};
use futures::future::poll_fn;
use std::{intrinsics::transmute, task::Poll};

type InitialStepFunction<T> = unsafe extern "C" fn(
    stack: &mut AsyncStack<T>,
    continuation: ContinuationFunction<T, T>,
    environment: &mut u8,
) -> cps::Result;

const INITIAL_STACK_CAPACITY: usize = 64;

pub async fn from_closure<T, S>(closure: Arc<Closure<T>>) -> S {
    let mut trampoline: Option<(StepFunction<(), S>, ContinuationFunction<(), S>)> = None;
    let mut stack = AsyncStack::new(INITIAL_STACK_CAPACITY);

    poll_fn(move |context| {
        stack.set_context(context);

        if let Some((step, continue_)) = trampoline {
            unsafe { step(&mut stack, continue_) };
        } else {
            unsafe {
                let entry_function =
                    transmute::<_, InitialStepFunction<S>>(closure.entry_function());
                entry_function(&mut stack, resolve, &mut *(closure.payload() as *mut u8))
            };
        }

        if let Some(value) = stack.resolved_value() {
            value.into()
        } else {
            trampoline = Some(stack.resume());
            Poll::Pending
        }
    })
    .await
}

extern "C" fn resolve<T>(stack: &mut AsyncStack<T>, value: T) -> cps::Result {
    stack.resolve(value);

    cps::Result::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Number;

    extern "C" fn foo(
        stack: &mut AsyncStack,
        continue_: ContinuationFunction<Number>,
        environment: &mut f64,
    ) -> cps::Result {
        unsafe { continue_(stack, (*environment).into()) }
    }

    #[tokio::test]
    async fn convert_closure() {
        let value = 42.0;

        assert_eq!(
            from_closure::<_, Number>(Arc::new(Closure::new(foo as *const u8, value))).await,
            value.into()
        );
    }
}
