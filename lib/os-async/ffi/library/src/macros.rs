#[macro_export]
macro_rules! export_fn {
    (async fn $function_name:ident ($($argument_name:ident: $argument_type:ty),*) -> $output_type:ty $body:block) => {
        #[no_mangle]
        unsafe extern "C" fn $function_name(
            stack: &mut ffi::cps::AsyncStack,
            continue_: ffi::cps::ContinuationFunction<$output_type>,
            $($argument_name: $argument_type),*
        ) -> ffi::cps::Result {
            type OutputFuture = Pin<Box<dyn Future<Output = $output_type>>>;

            async fn create_future($($argument_name: $argument_type),*) -> $output_type  {
                $body
            }

            let mut future: OutputFuture = Box::pin(create_future($($argument_name),*));

            unsafe extern "C" fn resume(
                stack: &mut ffi::cps::AsyncStack,
                continue_: ffi::cps::ContinuationFunction<$output_type>,
            ) -> ffi::cps::Result {
                let mut future: OutputFuture = stack.restore().unwrap();

                match future.as_mut().poll(stack.context().unwrap()) {
                    Poll::Ready(value) => continue_(stack, value),
                    Poll::Pending => {
                        stack.suspend(resume, continue_, future);
                        ffi::cps::Result::new()
                    }
                }
            }

            match future.as_mut().poll(stack.context().unwrap()) {
                Poll::Ready(value) => continue_(stack, value),
                Poll::Pending => {
                    stack.suspend(resume, continue_, future);
                    ffi::cps::Result::new()
                }
            }
        }
    };
}
