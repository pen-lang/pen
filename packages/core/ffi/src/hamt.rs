use core::hash::{Hash, Hasher};
use siphasher::sip::SipHasher;

#[ffi::bindgen]
fn _pen_core_hamt_hash_key(hash: ffi::Number, layer: ffi::Number) -> ffi::Number {
    let mut hasher = SipHasher::default();

    f64::from(hash).to_bits().hash(&mut hasher);
    (f64::from(layer) as u64).hash(&mut hasher);

    // Transmute a hash into f64 to return it in a proper register.
    f64::from_bits(hasher.finish()).into()
}

#[ffi::bindgen]
fn _pen_core_hamt_hash_string(string: ffi::ByteString) -> ffi::Number {
    let mut hasher = SipHasher::default();

    string.hash(&mut hasher);

    f64::from_bits(hasher.finish()).into()
}

#[ffi::bindgen]
fn _pen_core_hamt_hash_number(number: ffi::Number) -> ffi::Number {
    let mut hasher = SipHasher::default();

    f64::from(number).to_bits().hash(&mut hasher);

    f64::from_bits(hasher.finish()).into()
}
