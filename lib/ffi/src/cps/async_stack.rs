use super::Stack;
use std::{
    future::Future,
    ops::{Deref, DerefMut},
    task::Context,
};

#[repr(C)]
pub struct AsyncStack<'a> {
    stack: Stack,
    context: &'a mut Context<'a>,
    suspended: bool,
}

impl<'a> AsyncStack<'a> {
    pub fn new(capacity: usize, context: &'a mut Context<'a>) -> Self {
        Self {
            stack: Stack::new(capacity),
            context,
            suspended: false,
        }
    }

    pub fn context(&mut self) -> &mut Context<'a> {
        self.context
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

impl<'a> Deref for AsyncStack<'a> {
    type Target = Stack;

    fn deref(&self) -> &Stack {
        &self.stack
    }
}

impl<'a> DerefMut for AsyncStack<'a> {
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
        let mut stack = AsyncStack::new(16, &mut context);

        stack.push(42.0f64);

        assert_eq!(stack.pop::<f64>(), 42.0);
    }
}
