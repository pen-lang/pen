use super::Stack;
use std::{
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    task::Context,
};

#[repr(C)]
pub struct AsyncStack<'a, 'b> {
    stack: Stack,
    context: *mut Context<'b>,
    suspended: bool,
    _context: PhantomData<&'a Context<'b>>,
}

impl<'a, 'b> AsyncStack<'a, 'b> {
    pub fn new(capacity: usize, context: &'a mut Context<'b>) -> Self {
        Self {
            stack: Stack::new(capacity),
            context: context as *mut Context<'b> as *mut Context<'b>,
            suspended: false,
            _context: Default::default(),
        }
    }

    pub fn context(&mut self) -> &mut Context<'b> {
        unsafe { &mut *(self.context as *mut Context<'b>) }
    }

    pub fn suspend(&mut self, future: impl Future) {
        self.stack.push(future);
        self.suspended = true;
    }

    pub fn resume<F: Future>(&mut self) -> Option<F> {
        if self.suspended {
            self.suspended = false;
            Some(self.stack.pop())
        } else {
            None
        }
    }
}

impl<'a, 'b> Deref for AsyncStack<'a, 'b> {
    type Target = Stack;

    fn deref(&self) -> &Stack {
        &self.stack
    }
}

impl<'a, 'b> DerefMut for AsyncStack<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ptr::null,
        task::{RawWaker, RawWakerVTable, Waker},
    };

    const TEST_CAPACITY: usize = 1;
    const RAW_WAKER_DATA: () = ();
    const RAW_WAKER_V_TABLE: RawWakerVTable =
        RawWakerVTable::new(clone_waker, do_nothing, do_nothing, do_nothing);

    fn create_waker() -> Waker {
        unsafe { Waker::from_raw(RawWaker::new(&RAW_WAKER_DATA, &RAW_WAKER_V_TABLE)) }
    }

    fn clone_waker(_: *const ()) -> RawWaker {
        RawWaker::new(null(), &RAW_WAKER_V_TABLE)
    }

    fn do_nothing(_: *const ()) {}

    #[test]
    fn push_f64() {
        let waker = create_waker();
        let mut context = Context::from_waker(&waker);
        let mut stack = AsyncStack::new(TEST_CAPACITY, &mut context);

        stack.push(42.0f64);

        assert_eq!(stack.pop::<f64>(), 42.0);
    }

    #[test]
    fn wake() {
        let waker = create_waker();
        let mut context = Context::from_waker(&waker);
        let mut stack = AsyncStack::new(TEST_CAPACITY, &mut context);

        stack.context().waker().wake_by_ref();
    }
}
