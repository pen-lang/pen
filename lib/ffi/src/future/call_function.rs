#[macro_export]
macro_rules! call_function {
    (fn($($argument_type:ty),* $(,)?) -> $result_type:ty, $function:expr) => {
        call_function!(fn($($argument_type),*) -> $result_type, $function,)
    };
    (fn($($argument_type:ty),* $(,)?) -> $result_type:ty, $function:expr, $($argument:expr),* $(,)?) => {
        async {
            use core::task::Poll;
            use $crate::future::__private::{INITIAL_STACK_CAPACITY, poll_fn};
            use $crate::cps;

            type AsyncStack = cps::AsyncStack<$result_type>;

            type Trampoline = cps::Trampoline<$result_type, $result_type>;

            extern "C" fn resolve(stack: &mut AsyncStack, value: $result_type) -> cps::Result {
                stack.resolve(value);

                cps::Result::new()
            }

            // Move arguments into an initializer function.
            let mut initialize = Some(|stack: &mut AsyncStack| {
                let function = $function;

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
