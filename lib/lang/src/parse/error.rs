use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq)]
pub struct ParseError {
    source_name: String,
    details: String,
}

impl ParseError {
    pub fn new(source_name: &str, errors: &impl std::error::Error) -> Self {
        Self {
            source_name: source_name.into(),
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
            self.source_name, self.details
        )
    }
}
