use ffi::AnyLike;
use std::sync::Arc;

#[repr(C)]
#[derive(Clone)]
pub struct StringArray {
    inner: ffi::Any,
}

impl StringArray {
    pub fn new(vector: Vec<ffi::ByteString>) -> Self {
        Self {
            inner: StringArrayInner::new(vector).into_any(),
        }
    }

    pub fn get(&self, index: usize) -> Option<ffi::ByteString> {
        StringArrayInner::from_any(self.inner.clone())
            .unwrap()
            .get(index)
    }

    pub fn len(&self) -> usize {
        StringArrayInner::from_any(self.inner.clone())
            .unwrap()
            .len()
    }
}

impl Default for StringArray {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl<T: Into<ffi::ByteString>> From<Vec<T>> for StringArray {
    fn from(vector: Vec<T>) -> Self {
        Self::new(vector.into_iter().map(|x| x.into()).collect())
    }
}

#[allow(clippy::redundant_allocation)]
#[derive(Clone)]
struct StringArrayInner {
    vector: Arc<Arc<[ffi::ByteString]>>,
}

ffi::type_information!(array_inner, crate::string_array::StringArrayInner);

impl StringArrayInner {
    pub fn new(vector: Vec<ffi::ByteString>) -> Self {
        Self {
            vector: Arc::new(Arc::from(vector)),
        }
    }

    pub fn get(&self, index: usize) -> Option<ffi::ByteString> {
        self.vector.get(index).cloned()
    }

    pub fn len(&self) -> usize {
        self.vector.len()
    }
}

#[no_mangle]
extern "C" fn _pen_ffi_string_array_get(
    array: ffi::Arc<StringArray>,
    index: ffi::Number,
) -> ffi::ByteString {
    array.get(f64::from(index) as usize - 1).unwrap_or_default()
}

#[no_mangle]
extern "C" fn _pen_ffi_string_array_length(array: ffi::Arc<StringArray>) -> ffi::Number {
    (array.len() as f64).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{drop, forget, size_of};

    mod string_array {
        use super::*;

        #[test]
        fn clone() {
            let x = StringArray::from(vec!["foo"]);

            forget(x.clone());
            forget(x);
        }

        #[test]
        fn drop_() {
            let x = StringArray::from(vec!["foo"]);

            drop(x.clone());
            drop(x);
        }

        #[test]
        fn get_element() {
            _pen_ffi_string_array_get(StringArray::from(vec!["foo"]).into(), 1.0.into());
        }
    }

    mod string_array_inner {
        use super::*;

        #[test]
        fn is_small_enough() {
            assert!(size_of::<StringArrayInner>() <= size_of::<usize>());
        }

        #[test]
        fn drop_() {
            let x = StringArrayInner::new(vec!["foo".into()]);

            drop(x.clone());
            drop(x);
        }

        #[test]
        fn get_element() {
            assert_eq!(
                StringArrayInner::new(vec!["foo".into()]).get(0),
                Some("foo".into())
            );
        }
    }
}
