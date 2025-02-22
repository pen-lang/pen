mod debug;
mod heap;
mod unreachable;
mod utilities;

const INITIAL_STACK_CAPACITY: usize = 256;

#[cfg(not(test))]
#[link(name = "main")]
unsafe extern "C" {
    fn _pen_main(
        stack: *mut ffi::cps::Stack,
        continuation: extern "C" fn(*mut ffi::cps::Stack, ffi::None),
    );
}

#[cfg(test)]
unsafe extern "C" fn _pen_main(
    _: *mut ffi::cps::Stack,
    _: extern "C" fn(*mut ffi::cps::Stack, ffi::None),
) {
}

fn main() {
    let mut stack = ffi::cps::Stack::new(INITIAL_STACK_CAPACITY);

    unsafe { _pen_main(&mut stack, do_nothing) };
}

extern "C" fn do_nothing(_: *mut ffi::cps::Stack, _: ffi::None) {}
