use crate::{
    cps::{self, AsyncStack, ContinuationFunction, StepFunction},
    Arc, Closure,
};
use core::{intrinsics::transmute, task::Poll};
use futures::future::poll_fn;

type InitialStepFunction<T, V> = extern "C" fn(
    stack: &mut AsyncStack<T>,
    continuation: ContinuationFunction<T, T>,
    closure: Arc<Closure<V>>,
) -> cps::Result;

const INITIAL_STACK_CAPACITY: usize = 64;

pub async fn from_closure<T, V>(closure: Arc<Closure<T>>) -> V {
    let mut closure = Some(closure);
    let mut trampoline: Option<(StepFunction<(), V>, ContinuationFunction<(), V>)> = None;
    let mut stack = AsyncStack::new(INITIAL_STACK_CAPACITY);

    poll_fn(move |context| {
        stack.run_with_context(context, |stack| {
            if let Some((step, continue_)) = trampoline {
                step(stack, continue_);
            } else {
                let closure = closure.take().unwrap();
                let entry_function =
                    unsafe { transmute::<_, InitialStepFunction<V, T>>(closure.entry_function()) };

                entry_function(stack, resolve, closure);
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
        closure: Arc<Closure<f64>>,
    ) -> cps::Result {
        unsafe { continue_(stack, (*closure.payload()).into()) }
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
