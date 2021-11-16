use super::Stack;
use std::{
    future::{Future, Ready},
    ops::Deref,
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
        // TODO
        // self.push(future);
        self.suspended = true;
    }

    pub fn resume(&mut self) -> Option<impl Future> {
        if self.suspended {
            self.suspended = false;
            let future: Option<Ready<()>> = None;
            future
            // TODO
            // self.pop()
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
