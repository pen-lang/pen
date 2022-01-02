use std::{
    intrinsics::transmute,
    marker::PhantomData,
    ptr::drop_in_place,
    sync::atomic::{AtomicPtr, Ordering},
};

#[repr(C)]
pub struct Closure<F: Send, T = ()> {
    entry_function: AtomicPtr<u8>,
    drop_function: AtomicPtr<u8>,
    payload: T,
    _entry_function: PhantomData<F>,
}

impl<F: Send, T> Closure<F, T> {
    pub fn new(entry_function: F, payload: T) -> Self {
        Self {
            entry_function: AtomicPtr::new(unsafe { transmute(entry_function) }),
            drop_function: unsafe { transmute::<extern "C" fn(&mut Self), _>(drop_function) },
            payload,
            _entry_function: Default::default(),
        }
    }

    pub fn entry_function(&self) -> F {
        // TODO Optimize an atomic ordering.
        unsafe { transmute(self.entry_function.load(Ordering::SeqCst)) }
    }

    pub fn payload(&self) -> *const T {
        &self.payload
    }
}

extern "C" fn drop_function<F: Send, T>(closure: &mut Closure<F, T>) {
    unsafe { drop_in_place(&mut (closure.payload() as *mut T)) }
}

impl<F: Send, T> Drop for Closure<F, T> {
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
    use std::thread::spawn;

    fn foo() {}

    #[test]
    fn send() {
        let closure = Arc::new(Closure::new(foo, ()));

        spawn(move || {
            closure.entry_function();
        });
    }
}
