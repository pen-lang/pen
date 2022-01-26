use core::hash::{Hash, Hasher};
use siphasher::sip::SipHasher;

#[ffi::bindgen]
fn _pen_core_map_hash_key(key: ffi::ByteString) -> u64 {
    let mut hasher = SipHasher::default();

    key.as_slice().hash(&mut hasher);

    hasher.finish()
}
