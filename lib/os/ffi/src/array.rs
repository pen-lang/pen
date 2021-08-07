use ffi::AnyLike;
use std::sync::Arc;

#[repr(C)]
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

#[derive(Clone)]
struct ArrayInner {
    vector: Box<Arc<[ffi::Any]>>,
}

ffi::type_information!(array_inner, crate::array::ArrayInner);

impl ArrayInner {
    pub fn new(vector: Vec<ffi::Any>) -> Self {
        Self {
            vector: Box::new(vector.into()),
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
        .get(f64::from(index) as usize)
        .unwrap_or_else(ffi::Any::default)
}

#[no_mangle]
extern "C" fn _pen_ffi_array_length(array: ffi::Arc<Array>) -> ffi::Number {
    (array.len() as f64).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn array_inner_is_small_enough() {
        assert!(size_of::<ArrayInner>() <= size_of::<usize>());
    }
}
