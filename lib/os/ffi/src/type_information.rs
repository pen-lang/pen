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
            pub extern "C" fn clone(x: u64) {
                let x = unsafe { std::intrinsics::transmute::<_, $type>(x) };
                std::mem::forget(x.clone());
                std::mem::forget(x);
            }

            pub extern "C" fn drop(x: u64) {
                unsafe { std::intrinsics::transmute::<_, $type>(x) };
            }

            pub static type_information: crate::type_information::TypeInformation =
                crate::type_information::TypeInformation {
                    clone: $name::clone,
                    drop: $name::drop,
                };
        }

        impl $type {
            pub fn to_any(self) -> crate::type_information::Any {
                crate::type_information::Any::new(&$name::type_information, unsafe {
                    std::mem::transmute(self)
                })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    #[derive(Clone)]
    pub struct TypeA {
        value: std::rc::Rc<f64>,
    }

    #[derive(Clone)]
    pub struct TypeB {
        value: std::rc::Rc<std::rc::Rc<f64>>,
    }

    type_information!(foo, crate::type_information::tests::TypeA);
    type_information!(bar, crate::type_information::tests::TypeB);
}
