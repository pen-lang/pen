use crate::input::{position, Input};
use nom::error::{VerboseError, VerboseErrorKind};
use position::Position;
use std::{error::Error, fmt, fmt::Display};

pub type NomError<'a> = VerboseError<Input<'a>>;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    message: String,
    position: Position,
}

impl ParseError {
    pub fn new<'a>(source: &str, path: &str, error: nom::Err<NomError<'a>>) -> Self {
        match error {
            nom::Err::Incomplete(_) => Self::unexpected_end(source, path),
            nom::Err::Error(error) | nom::Err::Failure(error) => {
                if let Some(&(input, _)) = error.errors.first() {
                    Self {
                        message: [error
                            .errors
                            .iter()
                            .find_map(|(_, kind)| {
                                if let VerboseErrorKind::Char(character) = kind {
                                    Some(format!("letter '{}' expected", character))
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "failed to parse".into())]
                        .into_iter()
                        .chain(error.errors.iter().flat_map(|(_, kind)| {
                            if let VerboseErrorKind::Context(context) = kind {
                                Some(format!("in \"{}\"", context))
                            } else {
                                None
                            }
                        }))
                        .collect::<Vec<_>>()
                        .join(" "),
                        position: position(input),
                    }
                } else {
                    Self::unexpected_end(source, path)
                }
            }
        }
    }

    fn unexpected_end(source: &str, path: &str) -> Self {
        let lines = source.split('\n').collect::<Vec<_>>();
        let line = lines
            .iter()
            .rev()
            .find(|string| !string.is_empty())
            .map(|string| string.to_string())
            .unwrap_or_default();

        Self {
            message: "unexpected end of source".into(),
            position: Position::new(path, lines.len(), line.len(), line),
        }
    }
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            formatter,
            "{}",
            [self.message.as_str(), &self.position.to_string()].join("\n"),
        )
    }
}
