mod argument;
mod array;
mod directory;
mod environment_variable;
mod error;
mod file;
mod heap;
mod open_file_options;
mod result;
mod stdio;
mod utilities;

const INITIAL_STACK_CAPACITY: usize = 256;

#[cfg(not(test))]
#[link(name = "main")]
extern "C" {
    fn _pen_os_main(
        stack: *mut ffi::cps::Stack,
        continuation: extern "C" fn(*mut ffi::cps::Stack, f64) -> ffi::cps::Result,
    ) -> ffi::cps::Result;
}

#[cfg(test)]
unsafe extern "C" fn _pen_os_main(
    _: *mut ffi::cps::Stack,
    _: extern "C" fn(*mut ffi::cps::Stack, f64) -> ffi::cps::Result,
) -> ffi::cps::Result {
    ffi::cps::Result::new()
}

fn main() {
    let mut stack = ffi::cps::Stack::new(INITIAL_STACK_CAPACITY);

    unsafe { _pen_os_main(&mut stack, exit) };

    unreachable!()
}

extern "C" fn exit(_: *mut ffi::cps::Stack, code: f64) -> ffi::cps::Result {
    std::process::exit(code as i32)
}
