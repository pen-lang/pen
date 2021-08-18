#[no_mangle]
extern "C" fn _pen_convert_number_to_string(number: ffi::Number) -> ffi::ByteString {
    format!("{}", f64::from(number)).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_string() {
        assert_eq!(_pen_convert_number_to_string(42.0.into()), "42".into());
    }
}
