use std::marker::PhantomData;

#[repr(C)]
#[derive(Clone)]
pub struct Closure<I, O, F: Copy + Fn(I) -> O> {
    entry_pointer: F,
    drop_function: extern "C" fn(*mut u8),
    arity: usize,
    _input: PhantomData<I>,
    _output: PhantomData<O>,
}

impl<I, O, F: Copy + Fn(I) -> O> Closure<I, O, F> {
    pub fn new(entry_pointer: F, arity: usize) -> Self {
        Self {
            entry_pointer,
            drop_function: drop_nothing,
            arity,
            _input: Default::default(),
            _output: Default::default(),
        }
    }

    pub fn call(&self, arguments: I) -> O {
        (self.entry_pointer)(arguments)
    }
}

extern "C" fn drop_nothing(_: *mut u8) {}

impl<I, O, F: Copy + Fn(I) -> O> Drop for Closure<I, O, F> {
    fn drop(&mut self) {
        (self.drop_function)(self as *mut Self as *mut u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_closure() {
        fn foo(x: usize, y: usize) -> usize {
            x + y
        }

        let closure = Closure::<(usize, usize), usize, _>::new(foo, 2);

        assert_eq!(closure(42, 42), 84);
    }
}
