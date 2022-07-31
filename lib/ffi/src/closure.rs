use crate::Arc;
use core::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicPtr, Ordering},
};

struct ClosureMetadata<T> {
    drop: extern "C" fn(&mut Closure<T>),
    #[allow(dead_code)]
    synchronize: extern "C" fn(&mut Closure<T>),
}

#[repr(C)]
pub struct Closure<T = ()> {
    inner: Arc<ClosureInner<T>>,
}

#[repr(C)]
pub struct ClosureInner<T = ()> {
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
        Self {
            inner: ClosureInner {
                entry_function: AtomicPtr::new(entry_function as *mut u8),
                metadata: AtomicPtr::new(&Self::METADATA as *const _ as *mut _),
                payload: ManuallyDrop::new(payload),
            }
            .into(),
        }
    }

    pub fn entry_function(&self) -> *const u8 {
        self.inner.entry_function.load(Ordering::Relaxed)
    }

    pub fn payload(&self) -> *const T {
        self.inner.payload.deref()
    }
}

extern "C" fn drop_closure<T>(closure: &mut Closure<T>) {
    unsafe { ManuallyDrop::drop(&mut closure.inner.payload) }
}

// All closures created in Rust should implement Sync already.
extern "C" fn synchronize_closure<T>(_: &mut Closure<T>) {}

impl<T> Drop for Closure<T> {
    fn drop(&mut self) {
        let metadata = unsafe { &*self.inner.metadata.load(Ordering::Relaxed) };

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
