mod array;
mod error;
mod file;
mod heap;
mod open_file_options;
mod result;
mod stdio;
mod utilities;

#[cfg(not(test))]
const INITIAL_STACK_CAPACITY: usize = 256;

extern "C" {
    fn _pen_os_main(
        stack: *mut ffi::cps::Stack,
        continuation: extern "C" fn(*mut ffi::cps::Stack, f64) -> ffi::cps::Result,
        argument: ffi::None,
    ) -> ffi::cps::Result;
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> std::os::raw::c_int {
    let mut stack = ffi::cps::Stack::new(INITIAL_STACK_CAPACITY);

    unsafe { _pen_os_main(&mut stack, exit, ffi::None::new()) };

    unreachable!()
}

#[cfg(not(test))]
extern "C" fn exit(_: *mut ffi::cps::Stack, code: f64) -> ffi::cps::Result {
    std::process::exit(code as i32)
}
