use core::hash::{Hash, Hasher};
use siphasher::sip::SipHasher;

#[ffi::bindgen]
fn _pen_core_hamt_hash_key(key: ffi::ByteString, layer: ffi::Number) -> u64 {
    let mut hasher = SipHasher::default();

    key.hash(&mut hasher);
    (f64::from(layer) as u64).hash(&mut hasher);

    hasher.finish()
}
