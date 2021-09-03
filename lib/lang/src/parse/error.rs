use std::{error::Error, fmt, fmt::Display};

#[derive(Debug, PartialEq)]
pub struct ParseError {
    path: String,
    details: String,
}

impl ParseError {
    pub fn new(path: &str, errors: &impl Error) -> Self {
        Self {
            path: path.into(),
            details: format!("{}", errors),
        }
    }
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            formatter,
            "failed to parse module {}\n{}",
            self.path, self.details
        )
    }
}
