#![no_std]

extern crate alloc;

use alloc::string::ToString;
use core::hash::{Hash, Hasher};
use siphasher::sip::SipHasher;

const HASH_MULTIPLIER: u64 = 31;

#[ffi::bindgen]
fn _pen_prelude_debug_number(number: ffi::Number) -> ffi::ByteString {
    f64::from(number).to_string().into()
}

#[ffi::bindgen]
fn _pen_prelude_equal_strings(one: ffi::ByteString, other: ffi::ByteString) -> ffi::Boolean {
    (one.as_slice() == other.as_slice()).into()
}

#[ffi::bindgen]
fn _pen_prelude_combine_hashes(one: ffi::Number, other: ffi::Number) -> ffi::Number {
    f64::from_bits(
        f64::from(one)
            .to_bits()
            .wrapping_mul(HASH_MULTIPLIER)
            .wrapping_add(f64::from(other).to_bits()),
    )
    .into()
}

#[ffi::bindgen]
fn _pen_prelude_hash_number(number: ffi::Number) -> ffi::Number {
    let number = f64::from(number);

    // We normalize a floating point number before hash calculation.
    // https://internals.rust-lang.org/t/f32-f64-should-implement-hash/5436
    hash(
        &(if number == 0.0 {
            0.0
        } else if number.is_nan() {
            f64::NAN
        } else {
            number
        })
        .to_bits(),
    )
}

#[ffi::bindgen]
fn _pen_prelude_hash_string(string: ffi::ByteString) -> ffi::Number {
    hash(&string)
}

#[ffi::bindgen]
fn _pen_prelude_hash_to_index(
    hash: ffi::Number,
    layer: ffi::Number,
    level: ffi::Number,
) -> ffi::Number {
    ((((f64::from(_pen_prelude_combine_hashes(
        hash,
        _pen_prelude_hash_number(layer),
    ))
    .to_bits()
        >> (5 * (f64::from(level) as u64 - 1)))
        & 0b11111)
        + 1) as f64)
        .into()
}

fn hash(value: &impl Hash) -> ffi::Number {
    let mut hasher = SipHasher::new();

    value.hash(&mut hasher);

    f64::from_bits(hasher.finish()).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn debug_number() {
        assert_eq!(_pen_prelude_debug_number(42.0.into()), "42".into());
    }

    mod equal_string {
        use super::*;

        #[test]
        fn equal_empty_strings() {
            let string = ffi::ByteString::default();

            assert_eq!(
                _pen_prelude_equal_strings(string.clone(), string),
                true.into()
            );
        }

        #[test]
        fn equal_one_byte_strings() {
            let string = ffi::ByteString::from(vec![0u8]);

            assert_eq!(
                _pen_prelude_equal_strings(string.clone(), string),
                true.into()
            );
        }

        #[test]
        fn not_equal_one_byte_strings() {
            let one = ffi::ByteString::default();
            let other = vec![0u8].into();

            assert_eq!(_pen_prelude_equal_strings(one, other), false.into());
        }

        #[test]
        fn equal_text_strings() {
            const TEXT: &[u8] = "hello".as_bytes();

            let string = ffi::ByteString::from(TEXT);

            assert_eq!(
                _pen_prelude_equal_strings(string.clone(), string),
                true.into()
            );
        }

        #[test]
        fn not_equal_text_strings() {
            const TEXT: &[u8] = "hello".as_bytes();
            const OTHER_TEXT: &[u8] = "hell0".as_bytes();

            assert_eq!(
                _pen_prelude_equal_strings(TEXT.into(), OTHER_TEXT.into(),),
                false.into()
            );
        }
    }
}
