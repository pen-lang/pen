use combine::{easy, stream::position::SourcePosition};
use position::Position;
use std::{error::Error, fmt, fmt::Display};

#[derive(Debug, PartialEq)]
pub struct ParseError {
    expected: Vec<String>,
    unexpected: Vec<String>,
    messages: Vec<String>,
    position: Position,
}

impl ParseError {
    pub fn new(
        source: &str,
        path: &str,
        errors: combine::easy::Errors<char, &str, SourcePosition>,
    ) -> Self {
        Self {
            expected: errors
                .errors
                .iter()
                .filter_map(|error| match error {
                    easy::Error::Expected(info) => Some(info.to_string()),
                    _ => None,
                })
                .collect(),
            unexpected: errors
                .errors
                .iter()
                .filter_map(|error| match error {
                    easy::Error::Unexpected(info) => Some(info.to_string()),
                    _ => None,
                })
                .collect(),
            messages: errors
                .errors
                .iter()
                .filter_map(|error| match error {
                    easy::Error::Message(info) => Some(info.to_string()),
                    easy::Error::Other(error) => Some(error.to_string()),
                    _ => None,
                })
                .collect(),
            position: Position::new(
                path,
                errors.position.line as usize,
                errors.position.column as usize,
                source
                    .split("\n")
                    .nth(errors.position.line as usize - 1)
                    .unwrap_or_default(),
            ),
        }
    }
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            formatter,
            "{}",
            vec![
                Some(if self.messages.is_empty() {
                    "failed to parse module".into()
                } else {
                    self.messages.join(". ")
                }),
                if self.unexpected.is_empty() {
                    None
                } else {
                    Some(format!("unexpected: {}", self.unexpected.join(", ")))
                },
                if self.expected.is_empty() {
                    None
                } else {
                    Some(format!("expected: {}", self.expected.join(", ")))
                },
                Some(self.position.to_string()),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n"),
        )
    }
}
