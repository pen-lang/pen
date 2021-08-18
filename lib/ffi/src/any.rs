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
        (self.type_information.clone)(self.payload);

        Self {
            type_information: self.type_information,
            payload: self.payload,
        }
    }
}

impl Drop for Any {
    fn drop(&mut self) {
        (self.type_information.drop)(self.payload);
    }
}

pub trait AnyLike: Sized {
    fn into_any(self) -> Any;
    fn from_any(any: Any) -> Option<Self>;
    fn as_inner(any: &Any) -> Option<&Self>;
}

#[repr(C)]
pub struct TypeInformation {
    pub clone: extern "C" fn(u64),
    pub drop: extern "C" fn(u64),
}

#[macro_export]
macro_rules! type_information {
    ($name: ident, $type: ty) => {
        mod $name {
            unsafe fn transmute_to_payload<T>(data: T) -> u64 {
                let mut payload = 0;

                std::ptr::write(&mut payload as *mut u64 as *mut T, data);

                payload
            }

            unsafe fn transmute_from_payload<T>(payload: u64) -> T {
                std::ptr::read(&payload as *const u64 as *const T)
            }

            #[allow(clippy::forget_copy)]
            extern "C" fn clone(x: u64) {
                let x = unsafe { transmute_from_payload::<$type>(x) };
                std::mem::forget(x.clone());
                std::mem::forget(x);
            }

            extern "C" fn drop(x: u64) {
                unsafe { transmute_from_payload::<$type>(x) };
            }

            static TYPE_INFORMATION: $crate::TypeInformation =
                $crate::TypeInformation { clone, drop };

            impl $crate::AnyLike for $type {
                fn into_any(self) -> $crate::Any {
                    $crate::Any::new(&TYPE_INFORMATION, unsafe { transmute_to_payload(self) })
                }

                fn from_any(any: $crate::Any) -> Option<$type> {
                    if std::ptr::eq(any.type_information(), &TYPE_INFORMATION) {
                        let x = unsafe { transmute_from_payload(*any.payload()) };
                        std::mem::forget(any);
                        Some(x)
                    } else {
                        None
                    }
                }

                fn as_inner(any: &$crate::Any) -> Option<&$type> {
                    if std::ptr::eq(any.type_information(), &TYPE_INFORMATION) {
                        Some(unsafe { std::mem::transmute(any.payload()) })
                    } else {
                        None
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
        Dummy::default().into_any()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod rc {
        use super::*;
        use std::rc::Rc;

        #[derive(Clone)]
        pub struct TypeA {
            value: std::rc::Rc<f64>,
        }

        #[allow(clippy::redundant_allocation)]
        #[derive(Clone)]
        pub struct TypeB {
            value: std::rc::Rc<std::rc::Rc<f64>>,
        }

        type_information!(foo, crate::any::tests::rc::TypeA);
        type_information!(bar, crate::any::tests::rc::TypeB);

        #[test]
        fn drop_any() {
            TypeA {
                value: Rc::new(42.0),
            }
            .into_any();
        }

        #[test]
        fn clone_any() {
            let x = TypeA {
                value: Rc::new(42.0),
            }
            .into_any();

            drop(x.clone());
            drop(x)
        }

        #[test]
        fn as_inner() {
            let x = TypeA {
                value: Rc::new(42.0),
            }
            .into_any();

            let y: Option<&TypeA> = AnyLike::as_inner(&x);

            drop(y.clone());
        }
    }

    mod f64 {
        use super::*;

        #[derive(Clone)]
        pub struct Type {
            value: f64,
        }

        type_information!(foo, crate::any::tests::f64::Type);

        #[test]
        fn drop_any() {
            Type { value: 42.0 }.into_any();
        }

        #[test]
        fn clone_any() {
            let x = Type { value: 42.0 }.into_any();

            drop(x.clone());
            drop(x)
        }
    }
}
