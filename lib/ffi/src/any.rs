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
    use super::*;

    mod box_ {
        use super::*;

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
        use std::rc::Rc;

        #[pen_ffi_macro::any(crate = "crate")]
        #[derive(Clone)]
        pub struct TypeA {
            #[allow(dead_code)]
            value: std::rc::Rc<f64>,
        }

        #[pen_ffi_macro::any(crate = "crate")]
        #[allow(clippy::redundant_allocation)]
        #[derive(Clone)]
        pub struct TypeB {
            #[allow(dead_code)]
            value: std::rc::Rc<std::rc::Rc<f64>>,
        }

        #[test]
        fn drop_any() {
            let _ = Any::from(TypeA {
                value: Rc::new(42.0),
            });
        }

        #[test]
        fn clone_any() {
            let x = Any::from(TypeA {
                value: Rc::new(42.0),
            });

            drop(x.clone());
            drop(x)
        }

        #[test]
        fn as_inner() {
            let x = Any::from(TypeA {
                value: Rc::new(42.0),
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
}
