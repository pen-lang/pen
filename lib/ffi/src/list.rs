use crate::{import, Any, Arc, Closure};

import!(pen_ffi_list_create, fn() -> List);
import!(pen_ffi_list_lazy, fn(xs: Closure) -> List);
import!(pen_ffi_list_prepend, fn(x: Any, xs: List) -> List);

#[pen_ffi_macro::into_any(crate = "crate", fn = "pen_ffi_list_to_any")]
#[repr(C)]
#[derive(Clone)]
pub struct List(Arc<ListInner>);

#[repr(C)]
struct ListInner {
    node: Closure,
}

impl List {
    pub fn new() -> Self {
        unsafe { pen_ffi_list_create() }
    }

    pub fn prepend(self, x: impl Into<Any>) -> Self {
        unsafe { pen_ffi_list_prepend(x.into(), self) }
    }

    pub fn lazy(xs: Closure) -> Self {
        unsafe { pen_ffi_list_lazy(xs) }
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Into<Any>, I: IntoIterator<Item = T>> From<I> for List
where
    <I as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    fn from(xs: I) -> Self {
        let mut list = Self::new();

        for x in xs.into_iter().rev() {
            list = Self::prepend(list, x);
        }

        list
    }
}
