use rand::{thread_rng, Rng};

#[no_mangle]
extern "C" fn _pen_os_random_number() -> ffi::Number {
    thread_rng().gen::<f64>().into()
}
