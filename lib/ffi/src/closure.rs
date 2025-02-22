use crate::Arc;
use core::{
    mem::{ManuallyDrop, transmute},
    ops::Deref,
    sync::atomic::{AtomicPtr, Ordering},
};

struct ClosureMetadata<T> {
    drop: extern "C" fn(&mut ClosureInner<T>),
    #[allow(dead_code)]
    synchronize: extern "C" fn(&mut ClosureInner<T>),
}

#[repr(C)]
#[derive(Clone)]
pub struct Closure<T = ()>(Arc<ClosureInner<T>>);

#[repr(C)]
struct ClosureInner<T> {
    entry_function: AtomicPtr<u8>,
    metadata: AtomicPtr<ClosureMetadata<T>>,
    payload: ManuallyDrop<T>,
}

impl<T> Closure<T> {
    const METADATA: ClosureMetadata<T> = ClosureMetadata {
        drop: drop_closure::<T>,
        synchronize: synchronize_closure::<T>,
    };

    pub fn new(entry_function: *const u8, payload: T) -> Self {
        Self(
            ClosureInner {
                entry_function: AtomicPtr::new(entry_function as *mut u8),
                metadata: AtomicPtr::new(&Self::METADATA as *const _ as *mut _),
                payload: ManuallyDrop::new(payload),
            }
            .into(),
        )
    }

    pub fn entry_function(&self) -> *const u8 {
        self.0.entry_function.load(Ordering::Relaxed)
    }

    pub fn payload(&self) -> *const T {
        self.0.payload.deref()
    }

    pub fn into_opaque(self) -> Closure<()> {
        unsafe { transmute(self) }
    }
}

extern "C" fn drop_closure<T>(closure: &mut ClosureInner<T>) {
    unsafe { ManuallyDrop::drop(&mut closure.payload) }
}

// All closures created in Rust should implement Sync already.
extern "C" fn synchronize_closure<T>(_: &mut ClosureInner<T>) {}

impl<T> Drop for ClosureInner<T> {
    fn drop(&mut self) {
        let metadata = unsafe { &*self.metadata.load(Ordering::Relaxed) };

        (metadata.drop)(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::boxed::Box;
    use core::{ptr::null, sync::atomic::AtomicBool};

    fn spawn<T: Send + 'static>(_: impl (FnOnce() -> T) + Send + 'static) {}

    #[test]
    fn send() {
        let closure = Closure::new(null(), ());

        spawn(move || {
            closure.entry_function();
        });
    }

    #[test]
    fn drop_payload() {
        struct Foo {}

        static FLAG: AtomicBool = AtomicBool::new(false);

        impl Drop for Foo {
            fn drop(&mut self) {
                FLAG.store(true, Ordering::SeqCst);
            }
        }

        Closure::new(null(), Foo {});

        assert!(FLAG.load(Ordering::SeqCst));
    }

    #[test]
    fn drop_boxed_payload() {
        Closure::new(null(), Box::new(42.0));
    }
}
