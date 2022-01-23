use super::async_stack_action::AsyncStackAction;
use core::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum CpsError {
    UnexpectedAsyncStackAction(AsyncStackAction),
}

// TODO Implement std::error::Error when it is not `no_std`.

impl Display for CpsError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedAsyncStackAction(expected) => {
                write!(formatter, "invalid stack action (expected: {:?})", expected)
            }
        }
    }
}
