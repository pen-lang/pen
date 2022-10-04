use core::ptr::null;

#[repr(C)]
pub struct TypeInformation {
    clone_fn: extern "C" fn(u64) -> u64,
    drop_fn: extern "C" fn(u64),
    synchronize_fn: extern "C" fn(u64),
    extra: *const (),
}

impl TypeInformation {
    pub const fn new(
        clone_fn: extern "C" fn(u64) -> u64,
        drop_fn: extern "C" fn(u64),
        synchronize_fn: extern "C" fn(u64),
    ) -> Self {
        Self {
            clone_fn,
            drop_fn,
            synchronize_fn,
            extra: null(),
        }
    }

    pub fn clone_fn(&self) -> extern "C" fn(u64) -> u64 {
        self.clone_fn
    }

    pub fn drop_fn(&self) -> extern "C" fn(u64) {
        self.drop_fn
    }

    pub fn synchronize_fn(&self) -> extern "C" fn(u64) {
        self.synchronize_fn
    }
}

unsafe impl Sync for TypeInformation {}
