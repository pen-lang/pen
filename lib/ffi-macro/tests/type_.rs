use pen_ffi_macro::any;

#[any(crate = "pen_ffi")]
#[derive(Clone)]
struct ZeroSized {}

#[allow(dead_code)]
#[any(crate = "pen_ffi")]
#[derive(Clone)]
struct PointerSized {
    x: usize,
}
