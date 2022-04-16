// Note that this macro should not be called for non-thunk closures.
#[macro_export]
macro_rules! call {
    (fn($($argument_type:ty),*) -> $result_type:ty, $closure:expr, $($argument:expr),*) => {
        async {
            use core::{intrinsics::transmute, task::Poll};
            use futures::future::poll_fn;
            use $crate::{cps, Arc, Closure};

            const INITIAL_STACK_CAPACITY: usize = 64;

            type AsyncStack = cps::AsyncStack<$result_type>;

            type ContinuationFunction = cps::ContinuationFunction<$result_type, $result_type>;

            type Trampoline = cps::Trampoline<$result_type, $result_type>;

            type InitialStepFunction<C> = extern "C" fn(
                stack: &mut AsyncStack,
                continuation: ContinuationFunction,
                closure: Arc<Closure<C>>,
                $($argument_type),*
            ) -> cps::Result;

            extern "C" fn resolve(stack: &mut AsyncStack, value: $result_type) -> cps::Result {
                stack.resolve(value);

                cps::Result::new()
            }

            let mut trampoline: Option<Trampoline> = None;
            let mut stack = AsyncStack::new(INITIAL_STACK_CAPACITY);

            poll_fn(move |context| {
                stack.run_with_context(context, |stack| {
                    if let Some((step, continue_)) = trampoline {
                        step(stack, continue_);
                    } else {
                        unsafe {
                            transmute::<_, InitialStepFunction<_>>(
                                $closure.entry_function(),
                            )(stack, resolve, $closure.clone(), $($argument),*);
                        }
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
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        cps::{self, AsyncStack, ContinuationFunction},
        Arc, Closure, Number,
    };

    extern "C" fn thunk_entry_function(
        stack: &mut AsyncStack,
        continue_: ContinuationFunction<Number>,
        closure: Arc<Closure<f64>>,
    ) -> cps::Result {
        continue_(stack, unsafe { *closure.payload() }.into())
    }

    #[tokio::test]
    async fn convert_thunk() {
        let value = 42.0;

        assert_eq!(
            call!(
                fn() -> Number,
                Arc::new(Closure::new(thunk_entry_function as *const u8, value)),
            )
            .await,
            value.into()
        );
    }

    extern "C" fn closure_entry_function(
        stack: &mut AsyncStack,
        continue_: ContinuationFunction<Number>,
        _closure: Arc<Closure<()>>,
        x: Number,
    ) -> cps::Result {
        continue_(stack, x)
    }

    #[tokio::test]
    async fn convert_one_argument_closure() {
        let value = 42.0;

        assert_eq!(
            call!(
                fn(Number) -> Number,
                Arc::new(Closure::new(closure_entry_function as *const u8, ())),
                value.into()
            )
            .await,
            value.into()
        );
    }

    extern "C" fn closure_2_arity_entry_function(
        stack: &mut AsyncStack,
        continue_: ContinuationFunction<Number>,
        _closure: Arc<Closure<()>>,
        x: Number,
        y: Number,
    ) -> cps::Result {
        continue_(stack, (f64::from(x) + f64::from(y)).into())
    }

    #[tokio::test]
    async fn convert_two_argument_closure() {
        assert_eq!(
            call!(
                fn(Number, Number) -> Number,
                Arc::new(Closure::new(
                    closure_2_arity_entry_function as *const u8,
                    ()
                )),
                40.0.into(),
                2.0.into()
            )
            .await,
            42.0.into()
        );
    }
}
