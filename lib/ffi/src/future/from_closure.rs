use crate::{cps, Arc, Closure};
use futures::future::poll_fn;
use std::{intrinsics::transmute, task::Poll};

type Stack<O> = cps::AsyncStack<Option<O>>;
type InitialStepFunction<O> = unsafe extern "C" fn(
    stack: &mut Stack<O>,
    continuation: ContinuationFunction<O>,
    environment: &mut u8,
) -> cps::Result;
type StepFunction<O> = cps::StepFunction<O, Option<O>>;
type ContinuationFunction<O> = cps::ContinuationFunction<O, Option<O>>;

const INITIAL_STACK_CAPACITY: usize = 64;

pub async fn from_closure<T, O: Clone>(closure: Arc<Closure<T>>) -> O {
    let mut trampoline: Option<(StepFunction<O>, ContinuationFunction<O>)> = None;
    let mut stack = Stack::new(INITIAL_STACK_CAPACITY, None);

    poll_fn(move |context| {
        stack.set_context(context);

        if let Some((step, continue_)) = trampoline {
            unsafe { step(&mut stack, continue_) };
        } else {
            unsafe {
                let entry_function =
                    transmute::<_, InitialStepFunction<O>>(closure.entry_function());
                entry_function(
                    &mut stack,
                    store_result,
                    &mut *(closure.payload() as *mut u8),
                )
            };
        }

        if let Some(value) = stack.storage() {
            value.clone().into()
        } else {
            trampoline = Some(stack.resume());
            Poll::Pending
        }
    })
    .await
}

extern "C" fn store_result<O>(stack: &mut Stack<O>, value: O) -> cps::Result {
    *stack.storage_mut() = Some(value);

    cps::Result::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Number;

    extern "C" fn foo(
        stack: &mut Stack<Number>,
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
