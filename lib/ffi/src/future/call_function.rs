#[macro_export]
macro_rules! call_function {
    (fn($($argument_type:ty),* $(,)?) -> $result_type:ty, $function:expr) => {
        call_function!(fn($($argument_type),*) -> $result_type, $function,)
    };
    (fn($($argument_type:ty),* $(,)?) -> $result_type:ty, $function:expr, $($argument:expr),* $(,)?) => {
        async {
            use core::{future::poll_fn, task::Poll};
            use $crate::{cps, future::__private::INITIAL_STACK_CAPACITY};

            type AsyncStack = cps::AsyncStack<$result_type>;

            type Trampoline = cps::Trampoline<$result_type, $result_type>;

            extern "C" fn resolve(stack: &mut AsyncStack, value: $result_type) {
                stack.resolve(value);
            }

            // Move arguments into an initializer function.
            let mut initialize = Some(|stack: &mut AsyncStack| {
                let function = $function;

                #[allow(clippy::macro_metavars_in_unsafe)]
                unsafe { function(stack, resolve, $($argument),*) };
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
        ByteString, Number,
    };
    use core::future::ready;

    unsafe extern "C" fn get_number(
        stack: &mut AsyncStack<Number>,
        continue_: ContinuationFunction<Number, Number>,
    ) {
        continue_(stack, 42.0.into())
    }

    #[tokio::test]
    async fn call_with_no_argument() {
        assert_eq!(
            call_function!(fn() -> Number, get_number,).await,
            42.0.into()
        );
    }

    unsafe extern "C" fn pass_through_number(
        stack: &mut AsyncStack<Number>,
        continue_: ContinuationFunction<Number, Number>,
        x: Number,
    ) {
        continue_(stack, x)
    }

    #[tokio::test]
    async fn call_one_argument_closure() {
        let value = 42.0;

        assert_eq!(
            call_function!(fn(Number) -> Number, pass_through_number, value.into()).await,
            value.into()
        );
    }

    unsafe extern "C" fn add_numbers(
        stack: &mut AsyncStack<Number>,
        continue_: ContinuationFunction<Number, Number>,
        x: Number,
        y: Number,
    ) {
        continue_(stack, (f64::from(x) + f64::from(y)).into())
    }

    #[tokio::test]
    async fn call_two_argument_closure() {
        assert_eq!(
            call_function!(
                fn(Number, Number) -> Number,
                add_numbers,
                40.0.into(),
                2.0.into(),
            )
            .await,
            42.0.into()
        );
    }

    unsafe extern "C" fn get_number_with_suspension(
        stack: &mut AsyncStack<Number>,
        continue_: ContinuationFunction<Number, Number>,
    ) {
        fn step(stack: &mut AsyncStack<Number>, continue_: ContinuationFunction<Number, Number>) {
            continue_(stack, 42.0.into())
        }

        stack.suspend(step, continue_, ready(())).unwrap();

        // Wake immediately as we are waiting for nothing!
        stack.context().unwrap().waker().wake_by_ref();
    }

    #[tokio::test]
    async fn call_closure_with_suspension() {
        assert_eq!(
            call_function!(fn() -> Number, get_number_with_suspension,).await,
            42.0.into()
        );
    }

    unsafe extern "C" fn closure_entry_function_with_string(
        stack: &mut AsyncStack<ByteString>,
        continue_: ContinuationFunction<ByteString, ByteString>,
        x: ByteString,
    ) {
        continue_(stack, x)
    }

    #[tokio::test]
    async fn move_argument() {
        let value = "foo";

        assert_eq!(
            call_function!(
                fn(ByteString) -> ByteString,
                closure_entry_function_with_string,
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
            call_function!(
                fn(ByteString) -> ByteString,
                closure_entry_function_with_string,
                value.clone(),
            )
            .await,
            value
        );
    }
}
