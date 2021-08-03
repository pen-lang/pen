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

    pub fn payload(&self) -> u64 {
        self.payload
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

            impl $type {
                #[allow(unused)]
                pub unsafe fn into_any(self) -> $crate::Any {
                    $crate::Any::new(&TYPE_INFORMATION, transmute_to_payload(self))
                }

                #[allow(unused)]
                pub unsafe fn from_any(any: $crate::Any) -> $type {
                    let x = transmute_from_payload(any.payload());
                    std::mem::forget(any);
                    x
                }
            }
        }
    };
}

#[derive(Clone, Default)]
struct Dummy {
    _dummy: u64,
}

type_information!(dummy, crate::any::Dummy);

impl Default for Any {
    fn default() -> Self {
        unsafe { Dummy::default().into_any() }
    }
}

#[cfg(test)]
mod tests {
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
            unsafe {
                TypeA {
                    value: Rc::new(42.0),
                }
                .into_any()
            };
        }

        #[test]
        fn clone_any() {
            let x = unsafe {
                TypeA {
                    value: Rc::new(42.0),
                }
                .into_any()
            };

            drop(x.clone());
            drop(x)
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
            unsafe { Type { value: 42.0 }.into_any() };
        }

        #[test]
        fn clone_any() {
            let x = unsafe { Type { value: 42.0 }.into_any() };

            drop(x.clone());
            drop(x)
        }
    }
}
