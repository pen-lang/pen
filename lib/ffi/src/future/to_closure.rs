use crate::{cps, Arc, Closure};
use std::{future::Future, pin::Pin, ptr, task::Poll};

type Stack<'a, O> = cps::AsyncStack<'a, Option<O>>;
type ContinuationFunction<O> = cps::ContinuationFunction<O, Option<O>>;

extern "C" fn poll<O, F: Future<Output = O>>(
    stack: &mut Stack<O>,
    continue_: ContinuationFunction<O>,
    mut future: Pin<Box<F>>,
) -> cps::Result {
    match future.as_mut().poll(stack.context().unwrap()) {
        Poll::Ready(value) => unsafe { continue_(stack, value) },
        Poll::Pending => {
            stack.suspend(resume::<O, F>, continue_, future);
            cps::Result::new()
        }
    }
}

extern "C" fn resume<O, F: Future<Output = O>>(
    stack: &mut Stack<O>,
    continue_: ContinuationFunction<O>,
) -> cps::Result {
    let future = stack.restore::<Pin<Box<F>>>().unwrap();

    poll(stack, continue_, future)
}

extern "C" fn get_result<O, F: Future<Output = O>>(
    stack: &mut Stack<O>,
    continue_: ContinuationFunction<O>,
    environment: *mut Pin<Box<F>>,
) -> cps::Result {
    poll(stack, continue_, unsafe { ptr::read(environment) })
}

pub fn to_closure<O, F: Future<Output = O>>(future: F) -> Arc<Closure<Pin<Box<F>>>> {
    Closure::new(get_result::<O, F> as *const u8, Box::pin(future)).into()
}
