use super::async_stack_action::AsyncStackAction;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, PartialEq)]
pub enum CpsError {
    UnexpectedAsyncStackAction(AsyncStackAction),
}

impl Error for CpsError {}

impl Display for CpsError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedAsyncStackAction(expected) => {
                write!(formatter, "invalid stack action (expected: {:?})", expected)
            }
        }
    }
}
