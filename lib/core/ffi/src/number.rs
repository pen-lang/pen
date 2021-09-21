#[no_mangle]
extern "C" fn _pen_core_convert_number_to_string(number: ffi::Number) -> ffi::ByteString {
    format!("{}", f64::from(number)).into()
}

#[no_mangle]
extern "C" fn _pen_core_remainder(x: ffi::Number, y: ffi::Number) -> ffi::Number {
    f64::from(x).rem_euclid(y.into()).into()
}

#[no_mangle]
extern "C" fn _pen_core_power(x: ffi::Number, y: ffi::Number) -> ffi::Number {
    // spell-checker: disable-next-line
    f64::from(x).powf(y.into()).into()
}

#[no_mangle]
extern "C" fn _pen_core_square_root(x: ffi::Number) -> ffi::Number {
    f64::from(x).sqrt().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_string() {
        assert_eq!(_pen_core_convert_number_to_string(42.0.into()), "42".into());
    }

    #[test]
    fn remainder() {
        assert_eq!(_pen_core_remainder(42.0.into(), 5.0.into()), 2.0.into());
    }

    #[test]
    fn power() {
        assert_eq!(_pen_core_power(2.0.into(), 3.0.into()), 8.0.into());
    }

    #[test]
    fn square_root() {
        assert_eq!(_pen_core_square_root(4.0.into()), 2.0.into());
    }
}
