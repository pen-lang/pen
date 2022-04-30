use crate::{Boolean, BoxAny, ByteString, Number};

extern "C" {
    fn _pen_ffi_any_is_boolean(any: BoxAny) -> Boolean;
    fn _pen_ffi_any_is_none(any: BoxAny) -> Boolean;
    fn _pen_ffi_any_is_number(any: BoxAny) -> Boolean;
    fn _pen_ffi_any_is_string(any: BoxAny) -> Boolean;

    fn _pen_ffi_any_to_boolean(any: BoxAny) -> Boolean;
    fn _pen_ffi_any_to_number(any: BoxAny) -> Number;
    fn _pen_ffi_any_to_string(any: BoxAny) -> ByteString;
}

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
        unsafe { _pen_ffi_any_is_boolean(self.clone().into()) }.into()
    }

    pub fn is_none(&self) -> bool {
        unsafe { _pen_ffi_any_is_none(self.clone().into()) }.into()
    }

    pub fn is_number(&self) -> bool {
        unsafe { _pen_ffi_any_is_number(self.clone().into()) }.into()
    }

    pub fn is_string(&self) -> bool {
        unsafe { _pen_ffi_any_is_string(self.clone().into()) }.into()
    }

    pub fn to_boolean(&self) -> Option<Boolean> {
        if self.is_boolean() {
            Some(unsafe { _pen_ffi_any_to_boolean(self.clone().into()) })
        } else {
            None
        }
    }

    pub fn to_number(&self) -> Option<Number> {
        if self.is_number() {
            Some(unsafe { _pen_ffi_any_to_number(self.clone().into()) })
        } else {
            None
        }
    }

    pub fn to_string(&self) -> Option<ByteString> {
        if self.is_string() {
            Some(unsafe { _pen_ffi_any_to_string(self.clone().into()) })
        } else {
            None
        }
    }
}

impl Clone for Any {
    fn clone(&self) -> Self {
        Self {
            type_information: self.type_information,
            payload: (self.type_information.clone)(self.payload),
        }
    }
}

impl Drop for Any {
    fn drop(&mut self) {
        (self.type_information.drop)(self.payload);
    }
}

#[repr(C)]
pub struct TypeInformation {
    pub clone: extern "C" fn(u64) -> u64,
    pub drop: extern "C" fn(u64),
}

#[pen_ffi_macro::any(crate = "crate")]
#[derive(Clone, Default)]
struct Dummy {}

impl Default for Any {
    fn default() -> Self {
        Dummy::default().into()
    }
}

#[cfg(test)]
mod tests {
    use crate::None;

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

    fn drop_send_and_sync(_: impl Send + Sync) {}

    #[test]
    fn implement_send_and_sync() {
        drop_send_and_sync(Any::from(None::new()));
    }
}
