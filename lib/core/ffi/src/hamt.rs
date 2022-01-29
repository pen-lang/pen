use core::hash::{Hash, Hasher};
use siphasher::sip::SipHasher;
use std::intrinsics::transmute;

#[ffi::bindgen]
fn _pen_core_hamt_hash_key(key: ffi::ByteString, layer: ffi::Number) -> ffi::Number {
    let mut hasher = SipHasher::default();

    key.hash(&mut hasher);
    (f64::from(layer) as u64).hash(&mut hasher);

    let hash = hasher.finish();
    // Transmute a hash into f64 to return it in a proper register.
    unsafe { transmute::<_, f64>(hash) }.into()
}
