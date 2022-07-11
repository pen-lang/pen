use alloc::format;

#[ffi::bindgen]
fn _pen_core_absolute(number: ffi::Number) -> ffi::Number {
    f64::from(number).abs().into()
}

#[ffi::bindgen]
fn _pen_core_ceil(number: ffi::Number) -> ffi::Number {
    f64::from(number).ceil().into()
}

#[ffi::bindgen]
fn _pen_core_convert_number_to_string(number: ffi::Number) -> ffi::ByteString {
    format!("{}", f64::from(number)).into()
}

#[ffi::bindgen]
fn _pen_core_floor(number: ffi::Number) -> ffi::Number {
    f64::from(number).floor().into()
}

#[ffi::bindgen]
fn _pen_core_infinity() -> ffi::Number {
    f64::INFINITY.into()
}

#[ffi::bindgen]
fn _pen_core_is_nan(x: ffi::Number) -> ffi::Boolean {
    f64::from(x).is_nan().into()
}

#[ffi::bindgen]
fn _pen_core_nan() -> ffi::Number {
    f64::NAN.into()
}

#[ffi::bindgen]
fn _pen_core_power(x: ffi::Number, y: ffi::Number) -> ffi::Number {
    // spell-checker: disable-next-line
    f64::from(x).powf(y.into()).into()
}

#[ffi::bindgen]
fn _pen_core_remainder(x: ffi::Number, y: ffi::Number) -> ffi::Number {
    f64::from(x).rem_euclid(y.into()).into()
}

#[ffi::bindgen]
fn _pen_core_round(number: ffi::Number) -> ffi::Number {
    f64::from(number).round().into()
}

#[ffi::bindgen]
fn _pen_core_square_root(x: ffi::Number) -> ffi::Number {
    f64::from(x).sqrt().into()
}

#[ffi::bindgen]
fn _pen_core_truncate(number: ffi::Number) -> ffi::Number {
    f64::from(number).trunc().into()
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
