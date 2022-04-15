use core::{
    intrinsics::transmute,
    ptr::drop_in_place,
    sync::atomic::{AtomicPtr, Ordering},
};

#[repr(C)]
pub struct Closure<T = ()> {
    entry_function: AtomicPtr<u8>,
    drop_function: AtomicPtr<u8>,
    payload: T,
}

impl<T> Closure<T> {
    pub fn new(entry_function: *const u8, payload: T) -> Self {
        Self {
            entry_function: AtomicPtr::new(entry_function as *mut u8),
            drop_function: unsafe { transmute::<extern "C" fn(&mut Self), _>(drop_function) },
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

extern "C" fn drop_function<T>(closure: &mut Closure<T>) {
    unsafe { drop_in_place(&mut (closure.payload() as *mut T)) }
}

impl<T> Drop for Closure<T> {
    fn drop(&mut self) {
        // TODO Optimize an atomic ordering.
        (unsafe {
            transmute::<_, extern "C" fn(&mut Self)>(self.drop_function.load(Ordering::SeqCst))
        })(self);
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
