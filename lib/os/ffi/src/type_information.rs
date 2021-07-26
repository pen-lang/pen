#[repr(C)]
pub struct TypeInformation {
    pub clone: extern "C" fn(u64),
    pub drop: extern "C" fn(u64),
}

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
        }

        #[no_mangle]
        static $name: crate::type_information::TypeInformation =
            crate::type_information::TypeInformation {
                clone: $name::clone,
                drop: $name::drop,
            };
    };
}

#[cfg(test)]
mod tests {
    type_information!(foo, std::rc::Rc<f64>);
    type_information!(bar, std::rc::Rc<std::rc::Rc<f64>>);
}
