mod call;
mod call_function;
mod from_closure;
mod from_function;
pub mod stream;
mod to_closure;

pub use from_closure::*;
pub use from_function::*;
pub use to_closure::*;

pub mod __private {
    pub use futures::future::poll_fn;

    pub const INITIAL_STACK_CAPACITY: usize = 64;
}
