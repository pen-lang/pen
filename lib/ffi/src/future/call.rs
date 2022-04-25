// Note that this macro should not be called for non-thunk closures.
#[macro_export]
macro_rules! call {
    (fn($($argument_type:ty),* $(,)?) -> $result_type:ty, $closure:expr) => {
        call!(fn($($argument_type),*) -> $result_type, $closure,)
    };
    (fn($($argument_type:ty),* $(,)?) -> $result_type:ty, $closure:expr, $($argument:expr),* $(,)?) => {
        async {
            use core::{intrinsics::transmute, task::Poll};
            use $crate::future::__private::{INITIAL_STACK_CAPACITY, poll_fn};
            use $crate::{cps, Arc, Closure};

            type AsyncStack = cps::AsyncStack<$result_type>;

            type ContinuationFunction = cps::ContinuationFunction<$result_type, $result_type>;

            type Trampoline = cps::Trampoline<$result_type, $result_type>;

            type InitialStepFunction<C> = extern "C" fn(
                stack: &mut AsyncStack,
                continuation: ContinuationFunction,
                closure: Arc<Closure<C>>,
                $($argument_type),*
            );

            extern "C" fn resolve(stack: &mut AsyncStack, value: $result_type) {
                stack.resolve(value);
            }

            // Move closure and arguments into an initializer function.
            let mut initialize = Some(|stack: &mut AsyncStack| {
                let closure = $closure;

                (unsafe {
                    transmute::<_, InitialStepFunction<_>>(closure.entry_function())
                })(stack, resolve, closure, $($argument),*);
            });
            let mut trampoline: Option<Trampoline> = None;
            let mut stack = AsyncStack::new(INITIAL_STACK_CAPACITY);

            poll_fn(move |context| {
                if let Some(initialize) = initialize.take() {
                    stack.run_with_context(context, initialize);
                } else if let Some((step, continue_)) = trampoline.take() {
                    stack.run_with_context(context, |stack| step(stack, continue_));
                } else {
                    unreachable!("suspension must return trampoline functions")
                }

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
        cps::{AsyncStack, ContinuationFunction},
        Arc, ByteString, Closure, Number,
    };
    use core::future::ready;

    extern "C" fn thunk_entry_function(
        stack: &mut AsyncStack,
        continue_: ContinuationFunction<Number>,
        closure: Arc<Closure<f64>>,
    ) {
        continue_(stack, unsafe { *closure.payload() }.into())
    }

    #[tokio::test]
    async fn call_thunk() {
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
    ) {
        continue_(stack, x)
    }

    #[tokio::test]
    async fn call_one_argument_closure() {
        let value = 42.0;

        assert_eq!(
            call!(
                fn(Number) -> Number,
                Arc::new(Closure::new(closure_entry_function as *const u8, ())),
                value.into(),
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
    ) {
        continue_(stack, (f64::from(x) + f64::from(y)).into())
    }

    #[tokio::test]
    async fn call_two_argument_closure() {
        assert_eq!(
            call!(
                fn(Number, Number) -> Number,
                Arc::new(Closure::new(
                    closure_2_arity_entry_function as *const u8,
                    ()
                )),
                40.0.into(),
                2.0.into(),
            )
            .await,
            42.0.into()
        );
    }

    type TestResult = Number;

    extern "C" fn closure_entry_function_with_suspension(
        stack: &mut AsyncStack<TestResult>,
        continue_: ContinuationFunction<TestResult, TestResult>,
        _closure: Arc<Closure<()>>,
    ) {
        fn step(
            stack: &mut AsyncStack<TestResult>,
            continue_: ContinuationFunction<TestResult, TestResult>,
        ) {
            continue_(stack, 42.0.into())
        }

        stack.suspend(step, continue_, ready(())).unwrap();

        // Wake immediately as we are waiting for nothing!
        stack.context().unwrap().waker().wake_by_ref();
    }

    #[tokio::test]
    async fn call_closure_with_suspension() {
        assert_eq!(
            call!(
                fn() -> Number,
                Arc::new(Closure::new(
                    closure_entry_function_with_suspension as *const u8,
                    ()
                )),
            )
            .await,
            42.0.into()
        );
    }

    extern "C" fn closure_entry_function_with_string(
        stack: &mut AsyncStack,
        continue_: ContinuationFunction<ByteString>,
        _closure: Arc<Closure<()>>,
        x: ByteString,
    ) {
        continue_(stack, x)
    }

    #[tokio::test]
    async fn move_argument() {
        let value = "foo";

        assert_eq!(
            call!(
                fn(ByteString) -> ByteString,
                Arc::new(Closure::new(
                    closure_entry_function_with_string as *const u8,
                    ()
                )),
                value.into(),
            )
            .await,
            value.into()
        );
    }

    #[tokio::test]
    async fn move_argument_in_closure() {
        let value = ByteString::from("foo");

        assert_eq!(
            call!(
                fn(ByteString) -> ByteString,
                Arc::new(Closure::new(
                    closure_entry_function_with_string as *const u8,
                    ()
                )),
                value.clone(),
            )
            .await,
            value
        );
    }
}
