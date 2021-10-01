mod heap;
mod test_result;
mod unreachable;

use test_result::TestResult;

extern "C" {
    fn _pen_test_convert_result(stack: ffi::Any) -> TestResult;
}

#[link(name = "main_test")]
extern "C" {
    fn _pen_test_example() -> ffi::Any;
}

fn main() {
    unsafe { _pen_test_convert_result(_pen_test_example()) };
}
