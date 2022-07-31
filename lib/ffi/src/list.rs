use crate::{Any, Arc, BoxAny, Closure};

extern "C" {
    fn pen_ffi_list_create() -> List;
    fn pen_ffi_list_lazy(xs: Closure) -> List;
    fn pen_ffi_list_prepend(x: BoxAny, xs: List) -> List;
}

#[pen_ffi_macro::any(crate = "crate")]
#[repr(C)]
#[derive(Clone)]
pub struct List {
    inner: Arc<ListInner>,
}

#[repr(C)]
struct ListInner {
    node: Closure,
}

impl List {
    pub fn new() -> Self {
        unsafe { pen_ffi_list_create() }
    }

    pub fn prepend(self, x: impl Into<Any>) -> Self {
        unsafe { pen_ffi_list_prepend(x.into().into(), self) }
    }

    pub fn lazy(xs: Closure) -> Self {
        unsafe { pen_ffi_list_lazy(xs) }
    }
}

impl Default for List {
    fn default() -> Self {
        List::new()
    }
}

impl<T: Into<Any>, I: IntoIterator<Item = T>> From<I> for List
where
    <I as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    fn from(xs: I) -> Self {
        let mut list = List::new();

        for x in xs.into_iter().rev() {
            list = List::prepend(list, x);
        }

        list
    }
}
