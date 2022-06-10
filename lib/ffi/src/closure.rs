use core::{
    ptr::drop_in_place,
    sync::atomic::{AtomicPtr, Ordering},
};

struct ClosureMetadata<T> {
    drop: extern "C" fn(&mut Closure<T>),
}

#[repr(C)]
pub struct Closure<T = ()> {
    entry_function: AtomicPtr<u8>,
    metadata: AtomicPtr<ClosureMetadata<T>>,
    payload: T,
}

impl<T> Closure<T> {
    const METADATA: ClosureMetadata<T> = ClosureMetadata {
        drop: drop_closure::<T>,
    };

    pub fn new(entry_function: *const u8, payload: T) -> Self {
        Self {
            entry_function: AtomicPtr::new(entry_function as *mut u8),
            metadata: AtomicPtr::new(&Self::METADATA as *const _ as *mut _),
            payload,
        }
    }

    pub fn entry_function(&self) -> *const u8 {
        // TODO Optimize an atomic ordering for non-thunk closures.
        self.entry_function.load(Ordering::SeqCst)
    }

    pub fn payload(&self) -> *const T {
        &self.payload
    }
}

extern "C" fn drop_closure<T>(closure: &mut Closure<T>) {
    unsafe { drop_in_place(&mut (closure.payload() as *mut T)) }
}

impl<T> Drop for Closure<T> {
    fn drop(&mut self) {
        let metadata = unsafe { &*self.metadata.load(Ordering::Relaxed) };

        (metadata.drop)(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arc;
    use core::ptr::null;

    fn spawn<T: Send + 'static>(_: impl (FnOnce() -> T) + Send + 'static) {}

    #[test]
    fn send() {
        let closure = Arc::new(Closure::new(null(), ()));

        spawn(move || {
            closure.entry_function();
        });
    }
}
