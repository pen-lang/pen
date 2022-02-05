#[no_mangle]
extern "C" fn _pen_os_exit(
    stack: &mut ffi::cps::AsyncStack<ffi::Number>,
    _: ffi::cps::ContinuationFunction<ffi::None>,
    code: ffi::Number,
) -> ffi::cps::Result {
    // Resolve a main function immediately with an exit code.
    stack.resolve(code);

    Default::default()
}
