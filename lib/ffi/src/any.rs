use crate::{Boolean, ByteString, Error, List, Number, TypeInformation, import};

import!(pen_ffi_any_is_boolean, fn(any: Any) -> Boolean);
import!(pen_ffi_any_is_error, fn(any: Any) -> Boolean);
import!(pen_ffi_any_is_none, fn(any: Any) -> Boolean);
import!(pen_ffi_any_is_list, fn(any: Any) -> Boolean);
import!(pen_ffi_any_is_number, fn(any: Any) -> Boolean);
import!(pen_ffi_any_is_string, fn(any: Any) -> Boolean);

import!(pen_ffi_any_to_boolean, fn(any: Any) -> Boolean);
import!(pen_ffi_any_to_error, fn(any: Any) -> Error);
import!(pen_ffi_any_to_list, fn(any: Any) -> List);
import!(pen_ffi_any_to_number, fn(any: Any) -> Number);
import!(pen_ffi_any_to_string, fn(any: Any) -> ByteString);

#[repr(C)]
pub struct Any {
    type_information: &'static TypeInformation,
    payload: u64,
}

impl Any {
    pub fn new(type_information: &'static TypeInformation, payload: u64) -> Self {
        Self {
            type_information,
            payload,
        }
    }

    pub fn type_information(&self) -> &'static TypeInformation {
        self.type_information
    }

    pub fn payload(&self) -> &u64 {
        &self.payload
    }

    pub fn is_boolean(&self) -> bool {
        unsafe { pen_ffi_any_is_boolean(self.clone()) }.into()
    }

    pub fn is_error(&self) -> bool {
        unsafe { pen_ffi_any_is_error(self.clone()) }.into()
    }

    pub fn is_none(&self) -> bool {
        unsafe { pen_ffi_any_is_none(self.clone()) }.into()
    }

    pub fn is_list(&self) -> bool {
        unsafe { pen_ffi_any_is_list(self.clone()) }.into()
    }

    pub fn is_number(&self) -> bool {
        unsafe { pen_ffi_any_is_number(self.clone()) }.into()
    }

    pub fn is_string(&self) -> bool {
        unsafe { pen_ffi_any_is_string(self.clone()) }.into()
    }
}

impl Clone for Any {
    fn clone(&self) -> Self {
        Self {
            type_information: self.type_information,
            payload: (self.type_information.clone_fn())(self.payload),
        }
    }
}

impl Drop for Any {
    fn drop(&mut self) {
        (self.type_information.drop_fn())(self.payload);
    }
}

impl TryFrom<Any> for Boolean {
    type Error = ();

    fn try_from(value: Any) -> Result<Self, ()> {
        if value.is_boolean() {
            Ok(unsafe { pen_ffi_any_to_boolean(value) })
        } else {
            Err(())
        }
    }
}

impl TryFrom<Any> for Error {
    type Error = ();

    fn try_from(value: Any) -> Result<Self, ()> {
        if value.is_error() {
            Ok(unsafe { pen_ffi_any_to_error(value) })
        } else {
            Err(())
        }
    }
}

impl TryFrom<Any> for List {
    type Error = ();

    fn try_from(value: Any) -> Result<Self, ()> {
        if value.is_list() {
            Ok(unsafe { pen_ffi_any_to_list(value) })
        } else {
            Err(())
        }
    }
}

impl TryFrom<Any> for Number {
    type Error = ();

    fn try_from(value: Any) -> Result<Self, ()> {
        if value.is_number() {
            Ok(unsafe { pen_ffi_any_to_number(value) })
        } else {
            Err(())
        }
    }
}

impl TryFrom<Any> for ByteString {
    type Error = ();

    fn try_from(value: Any) -> Result<Self, ()> {
        if value.is_string() {
            Ok(unsafe { pen_ffi_any_to_string(value) })
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod box_ {
        use super::*;
        use alloc::boxed::Box;

        #[pen_ffi_macro::any(crate = "crate")]
        #[derive(Clone)]
        pub struct TypeA {
            #[allow(dead_code)]
            value: Box<f64>,
        }

        #[pen_ffi_macro::any(crate = "crate")]
        #[allow(clippy::redundant_allocation)]
        #[derive(Clone)]
        pub struct TypeB {
            #[allow(dead_code)]
            value: Box<Box<f64>>,
        }

        #[test]
        fn drop_any() {
            let _ = Any::from(TypeA {
                value: Box::new(42.0),
            });
        }

        #[test]
        fn clone_any() {
            let x = Any::from(TypeA {
                value: Box::new(42.0),
            });

            drop(x.clone());
            drop(x)
        }

        #[test]
        fn as_inner() {
            let x = Any::from(TypeA {
                value: Box::new(42.0),
            });

            let _: &TypeA = (&x).try_into().unwrap();
        }
    }

    mod rc {
        use super::*;
        use alloc::sync::Arc;

        #[pen_ffi_macro::any(crate = "crate")]
        #[derive(Clone)]
        pub struct TypeA {
            #[allow(dead_code)]
            value: Arc<f64>,
        }

        #[pen_ffi_macro::any(crate = "crate")]
        #[allow(clippy::redundant_allocation)]
        #[derive(Clone)]
        pub struct TypeB {
            #[allow(dead_code)]
            value: Arc<Arc<f64>>,
        }

        #[test]
        fn drop_any() {
            let _ = Any::from(TypeA {
                value: Arc::new(42.0),
            });
        }

        #[test]
        fn clone_any() {
            let x = Any::from(TypeA {
                value: Arc::new(42.0),
            });

            drop(x.clone());
            drop(x)
        }

        #[test]
        fn as_inner() {
            let x = Any::from(TypeA {
                value: Arc::new(42.0),
            });

            let _: &TypeA = (&x).try_into().unwrap();
        }
    }

    mod f64 {
        use super::*;

        #[pen_ffi_macro::any(crate = "crate")]
        #[derive(Clone)]
        pub struct Type {
            #[allow(dead_code)]
            value: f64,
        }

        #[test]
        fn drop_any() {
            let _ = Any::from(Type { value: 42.0 });
        }

        #[test]
        fn clone_any() {
            let x = Any::from(Type { value: 42.0 });

            drop(x.clone());
            drop(x)
        }
    }

    mod send_sync {
        use super::*;

        #[pen_ffi_macro::any(crate = "crate")]
        #[derive(Clone, Default)]
        struct Dummy {}

        fn drop_send_and_sync(_: impl Send + Sync) {}

        #[test]
        fn implement_send_and_sync() {
            drop_send_and_sync(Any::from(Dummy::default()));
        }
    }
}
