#[macro_export]
macro_rules! import {
    ($name:ident, fn($($argument_name:ident: $argument_type:ty),* $(,)?) -> $result_type:ty $(,)?) => {
        unsafe extern "C" {
            fn $name($($argument_name: $argument_type),*) -> $result_type;
        }
    };
    ($name:ident, async fn($($argument_name:ident: $argument_type:ty),* $(,)?) -> $result_type:ty $(,)?) => {
        async fn $name($($argument_name: $argument_type),*) -> $result_type {
            unsafe extern "C" {
                // It is fine for AsyncStack to be FFI-unsafe because the compiler touches only its Stack part.
                #[allow(improper_ctypes)]
                fn $name(
                    stack: &mut $crate::cps::AsyncStack<$result_type>,
                    continue_: $crate::cps::ContinuationFunction<$result_type, $result_type>,
                    $($argument_name: $argument_type),*
                );
            }

            $crate::call_function!(fn($($argument_type),*) -> $result_type, $name, $($argument_name),*).await
        }
    };
}
