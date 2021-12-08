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

#[macro_export]
macro_rules! type_information {
    ($name: ident, $type: ty) => {
        mod $name {
            unsafe fn transmute_into_payload<T>(data: T) -> u64 {
                let mut payload = 0;

                std::ptr::write(&mut payload as *mut u64 as *mut T, data);

                payload
            }

            unsafe fn transmute_from_payload<T>(payload: u64) -> T {
                std::ptr::read(&payload as *const u64 as *const T)
            }

            #[allow(clippy::forget_copy)]
            extern "C" fn clone(x: u64) -> u64 {
                let x = unsafe { transmute_from_payload::<$type>(x) };
                let payload = unsafe { transmute_into_payload(x.clone()) };

                std::mem::forget(x);

                payload
            }

            extern "C" fn drop(x: u64) {
                unsafe { transmute_from_payload::<$type>(x) };
            }

            static TYPE_INFORMATION: $crate::TypeInformation =
                $crate::TypeInformation { clone, drop };

            impl From<$type> for $crate::Any {
                fn from(x: $type) -> Self {
                    Self::new(&TYPE_INFORMATION, unsafe { transmute_into_payload(x) })
                }
            }

            impl TryFrom<$crate::Any> for $type {
                type Error = ();

                fn try_from(any: $crate::Any) -> Result<Self, ()> {
                    if std::ptr::eq(any.type_information(), &TYPE_INFORMATION) {
                        let x = unsafe { transmute_from_payload(*any.payload()) };
                        std::mem::forget(any);
                        Ok(x)
                    } else {
                        Err(())
                    }
                }
            }

            impl<'a> TryFrom<&'a $crate::Any> for &'a $type {
                type Error = ();

                fn try_from(any: &$crate::Any) -> Result<Self, ()> {
                    if std::ptr::eq(any.type_information(), &TYPE_INFORMATION) {
                        Ok(unsafe { std::mem::transmute(any.payload()) })
                    } else {
                        Err(())
                    }
                }
            }
        }
    };
}

#[derive(Clone, Default)]
struct Dummy {}

type_information!(dummy, crate::any::Dummy);

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

        #[derive(Clone)]
        pub struct TypeA {
            #[allow(dead_code)]
            value: Box<f64>,
        }

        #[allow(clippy::redundant_allocation)]
        #[derive(Clone)]
        pub struct TypeB {
            #[allow(dead_code)]
            value: Box<Box<f64>>,
        }

        type_information!(foo, crate::any::tests::box_::TypeA);
        type_information!(bar, crate::any::tests::box_::TypeB);

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

        #[derive(Clone)]
        pub struct TypeA {
            #[allow(dead_code)]
            value: std::rc::Rc<f64>,
        }

        #[allow(clippy::redundant_allocation)]
        #[derive(Clone)]
        pub struct TypeB {
            #[allow(dead_code)]
            value: std::rc::Rc<std::rc::Rc<f64>>,
        }

        type_information!(foo, crate::any::tests::rc::TypeA);
        type_information!(bar, crate::any::tests::rc::TypeB);

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

        #[derive(Clone)]
        pub struct Type {
            #[allow(dead_code)]
            value: f64,
        }

        type_information!(foo, crate::any::tests::f64::Type);

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
