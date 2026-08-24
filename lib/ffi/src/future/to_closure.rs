use crate::{
    Closure,
    cps::{AsyncStack, ContinuationFunction},
};
use alloc::boxed::Box;
use core::{future::Future, pin::Pin, task::Poll};

impl<T, F: Future<Output = T>> From<F> for Closure {
    fn from(future: F) -> Self {
        to_closure(future)
    }
}

pub fn to_closure<O, F: Future<Output = O>>(future: F) -> Closure {
    Closure::new(get_result::<O, F> as *const u8, Some(Box::pin(future))).into_opaque()
}

extern "C" fn get_result<O, F: Future<Output = O>>(
    stack: &mut AsyncStack,
    continue_: ContinuationFunction<O>,
    closure: Closure<Option<Pin<Box<F>>>>,
) {
    poll(
        stack,
        continue_,
        unsafe { &mut *(closure.payload() as *mut Option<Pin<Box<F>>>) }
            .take()
            .unwrap(),
    )
}

fn resume<O, F: Future<Output = O>>(stack: &mut AsyncStack, continue_: ContinuationFunction<O>) {
    let future = stack.restore::<Pin<Box<F>>>().unwrap();

    poll(stack, continue_, future)
}

fn poll<O, F: Future<Output = O>>(
    stack: &mut AsyncStack,
    continue_: ContinuationFunction<O>,
    mut future: Pin<Box<F>>,
) {
    match future.as_mut().poll(stack.context().unwrap()) {
        Poll::Ready(value) => {
            stack.trampoline(continue_, value).unwrap();
        }
        Poll::Pending => {
            stack.suspend(resume::<O, F>, continue_, future).unwrap();
        }
    }
}
