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
                if let Some((input, kind)) = error.errors.last() {
                    Self {
                        message: match kind {
                            VerboseErrorKind::Context(context) => {
                                format!("failed to parse {}", context)
                            }
                            VerboseErrorKind::Char(character) => {
                                format!("letter '{}' expected", character)
                            }
                            VerboseErrorKind::Nom(_) => "failed to parse".into(),
                        },
                        position: position(*input),
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
