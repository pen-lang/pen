#[macro_export]
macro_rules! import {
    ($name:ident, fn($($argument_name:ident: $argument_type:ty),* $(,)?) -> $result_type:ty $(,)?) => {
        fn $name(
            stack: &mut $crate::cps::AsyncStack<$result_type>,
            continue_: $crate::cps::ContinuationFunction<$result_type, $result_type>,
            $($argument_name: $argument_type),*
        );
    };
}
