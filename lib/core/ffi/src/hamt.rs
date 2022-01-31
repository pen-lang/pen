use core::hash::{Hash, Hasher};
use siphasher::sip::SipHasher;

#[ffi::bindgen]
fn _pen_core_hamt_hash_key(key: ffi::ByteString, layer: ffi::Number) -> ffi::Number {
    let mut hasher = SipHasher::default();

    key.hash(&mut hasher);
    (f64::from(layer) as u64).hash(&mut hasher);

    // Transmute a hash into f64 to return it in a proper register.
    f64::from_bits(hasher.finish()).into()
}
