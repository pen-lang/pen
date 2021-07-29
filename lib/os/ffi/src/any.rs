#[repr(C)]
pub struct Any {
    type_information: &'static TypeInformation,
    payload: u64,
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

            pub static TYPE_INFORMATION: crate::any::TypeInformation =
                crate::any::TypeInformation { clone, drop };

            impl From<$type> for crate::any::Any {
                fn from(payload: $type) -> crate::any::Any {
                    crate::any::Any::new(&TYPE_INFORMATION, unsafe { std::mem::transmute(payload) })
                }
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

    type_information!(foo, crate::any::tests::TypeA);
    type_information!(bar, crate::any::tests::TypeB);
}
