use pen_ffi_macro::any_conv;

#[any_conv(crate = "pen_ffi")]
#[derive(Clone)]
struct Foo {}
