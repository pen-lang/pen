use crate::{type_information, Any, AnyLike, ByteString};
use std::sync::Arc;

#[repr(C)]
#[derive(Clone)]
pub struct StringArray {
    inner: Any,
}

impl StringArray {
    pub fn new(vector: Vec<ByteString>) -> Self {
        Self {
            inner: StringArrayInner::new(vector).into_any(),
        }
    }

    pub fn get(&self, index: usize) -> Option<ByteString> {
        StringArrayInner::from_any(self.inner.clone())
            .unwrap()
            .get(index)
    }

    pub fn len(&self) -> usize {
        StringArrayInner::from_any(self.inner.clone())
            .unwrap()
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for StringArray {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl<T: Into<ByteString>> From<Vec<T>> for StringArray {
    fn from(vector: Vec<T>) -> Self {
        Self::new(vector.into_iter().map(|x| x.into()).collect())
    }
}

#[allow(clippy::redundant_allocation)]
#[derive(Clone)]
struct StringArrayInner {
    vector: Arc<Vec<ByteString>>,
}

type_information!(array_inner, crate::extra::string_array::StringArrayInner);

impl StringArrayInner {
    pub fn new(vector: Vec<ByteString>) -> Self {
        Self {
            vector: Arc::new(vector),
        }
    }

    pub fn get(&self, index: usize) -> Option<ByteString> {
        self.vector.get(index).cloned()
    }

    pub fn len(&self) -> usize {
        self.vector.len()
    }
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
            StringArray::from(vec!["foo"]).get(1);
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
