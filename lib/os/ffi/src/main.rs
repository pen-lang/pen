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

#[link(name = "main")]
extern "Rust" {
    fn _pen_os_main(
        stack: *mut ffi::cps::Stack,
        continuation: fn(*mut ffi::cps::Stack, f64) -> ffi::cps::Result,
    ) -> ffi::cps::Result;
}

fn main() {
    let mut stack = ffi::cps::Stack::new(INITIAL_STACK_CAPACITY);

    unsafe { _pen_os_main(&mut stack, exit) };

    unreachable!()
}

fn exit(_: *mut ffi::cps::Stack, code: f64) -> ffi::cps::Result {
    std::process::exit(code as i32)
}
