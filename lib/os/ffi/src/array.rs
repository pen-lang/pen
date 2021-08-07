use ffi::AnyLike;
use std::sync::Arc;

#[repr(C)]
#[derive(Clone)]
pub struct Array {
    inner: ffi::Any,
}

impl Array {
    pub fn new(vector: Vec<ffi::Any>) -> Self {
        Self {
            inner: ArrayInner::new(vector).into_any(),
        }
    }

    pub fn get(&self, index: usize) -> Option<ffi::Any> {
        ArrayInner::from_any(self.inner.clone()).unwrap().get(index)
    }

    pub fn len(&self) -> usize {
        ArrayInner::from_any(self.inner.clone()).unwrap().len()
    }
}

impl<T: ffi::AnyLike> From<Vec<T>> for Array {
    fn from(vector: Vec<T>) -> Self {
        Self::new(vector.into_iter().map(|x| x.into_any()).collect())
    }
}

#[allow(clippy::redundant_allocation)]
#[derive(Clone)]
struct ArrayInner {
    vector: Arc<Arc<[ffi::Any]>>,
}

ffi::type_information!(array_inner, crate::array::ArrayInner);

impl ArrayInner {
    pub fn new(vector: Vec<ffi::Any>) -> Self {
        Self {
            vector: Arc::new(vector.into()),
        }
    }

    pub fn get(&self, index: usize) -> Option<ffi::Any> {
        self.vector.get(index).cloned()
    }

    pub fn len(&self) -> usize {
        self.vector.len()
    }
}

#[no_mangle]
extern "C" fn _pen_ffi_array_get(array: ffi::Arc<Array>, index: ffi::Number) -> ffi::Any {
    array
        .get(f64::from(index) as usize - 1)
        .unwrap_or_else(ffi::Any::default)
}

#[no_mangle]
extern "C" fn _pen_ffi_array_length(array: ffi::Arc<Array>) -> ffi::Number {
    (array.len() as f64).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        mem::{drop, forget, size_of},
        rc::Rc,
    };

    #[derive(Clone, Debug, PartialEq)]
    struct Foo {
        x: Rc<f64>,
    }

    ffi::type_information!(foo, crate::array::tests::Foo);

    mod array {
        use super::*;

        #[test]
        fn clone() {
            let x = Array::from(vec![Foo { x: 42.0.into() }]);

            forget(x.clone());
            forget(x);
        }

        #[test]
        fn drop_() {
            let x = Array::from(vec![Foo { x: 42.0.into() }]);

            drop(x.clone());
            drop(x);
        }

        #[test]
        fn get_element() {
            _pen_ffi_array_get(Array::from(vec![Foo { x: 42.0.into() }]).into(), 1.0.into());
        }
    }

    mod array_inner {
        use super::*;

        #[test]
        fn is_small_enough() {
            assert!(size_of::<ArrayInner>() <= size_of::<usize>());
        }

        #[test]
        fn drop_() {
            let x = ArrayInner::new(vec![Foo { x: 42.0.into() }.into_any()]);

            drop(x.clone());
            drop(x);
        }

        #[test]
        fn get_element() {
            assert_eq!(
                Foo::from_any(
                    ArrayInner::new(vec![Foo { x: 42.0.into() }.into_any()])
                        .get(0)
                        .unwrap()
                )
                .unwrap(),
                Foo { x: 42.0.into() }
            );
        }
    }
}
