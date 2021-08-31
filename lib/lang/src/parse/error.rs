
use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq)]
pub struct ParseError {
    path: String,
    details: String,
}

impl ParseError {
    pub fn new(path: &str, errors: &impl std::error::Error) -> Self {
        Self {
            path: path.into(),
            details: format!("{}", errors),
        }
    }
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            formatter,
            "failed to parse module {}\n{}",
            self.path, self.details
        )
    }
}
