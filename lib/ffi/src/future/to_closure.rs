use crate::{
    cps::{self, AsyncStack, ContinuationFunction},
    Arc, Closure,
};
use std::{future::Future, intrinsics::transmute, pin::Pin, task::Poll};

impl<T, F: Future<Output = T>> From<F> for Arc<Closure> {
    fn from(future: F) -> Self {
        to_closure(future)
    }
}

pub fn to_closure<O, F: Future<Output = O>>(future: F) -> Arc<Closure> {
    let closure = Arc::new(Closure::new(
        get_result::<O, F> as *const u8,
        Some(Box::pin(future)),
    ));

    unsafe { transmute(closure) }
}

extern "C" fn get_result<O, F: Future<Output = O>>(
    stack: &mut AsyncStack,
    continue_: ContinuationFunction<O>,
    closure: Arc<Closure<Option<Pin<Box<F>>>>>,
) -> cps::Result {
    poll(
        stack,
        continue_,
        unsafe { &mut *(closure.payload() as *mut Option<Pin<Box<F>>>) }
            .take()
            .unwrap(),
    )
}

extern "C" fn resume<O, F: Future<Output = O>>(
    stack: &mut AsyncStack,
    continue_: ContinuationFunction<O>,
) -> cps::Result {
    let future = stack.restore::<Pin<Box<F>>>().unwrap();

    poll(stack, continue_, future)
}

extern "C" fn poll<O, F: Future<Output = O>>(
    stack: &mut AsyncStack,
    continue_: ContinuationFunction<O>,
    mut future: Pin<Box<F>>,
) -> cps::Result {
    match future.as_mut().poll(stack.context().unwrap()) {
        Poll::Ready(value) => continue_(stack, value),
        Poll::Pending => {
            stack.suspend(resume::<O, F>, continue_, future).unwrap();
            cps::Result::new()
        }
    }
}
