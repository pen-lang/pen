use pen_ffi_macro::{any, into_any};

#[any(crate = "pen_ffi")]
#[derive(Clone)]
struct ZeroSized {}

#[allow(dead_code)]
#[any(crate = "pen_ffi")]
#[derive(Clone)]
struct PointerSized {
    x: usize,
}

#[into_any(crate = "pen_ffi", into_fn = "foo_to_any")]
#[repr(C)]
struct Foo {
    x: usize,
}
