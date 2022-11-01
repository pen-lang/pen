mod combinator;
mod error;
mod operations;
mod parser;

use ast::Comment;
pub use error::ParseError;
use parser::{comments, input, module};

pub fn parse(source: &str, path: &str) -> Result<ast::Module, ParseError> {
    module(input(source, path))
        .map(|(_, module)| module)
        .map_err(|error| ParseError::new(source, path, error))
}

pub fn parse_comments(source: &str, path: &str) -> Result<Vec<Comment>, ParseError> {
    comments(input(source, path))
        .map(|(_, comments)| comments)
        .map_err(|error| ParseError::new(source, path, error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::*;
    use position::{test::PositionFake, Position};

    #[test]
    fn parse_empty_module() {
        assert_eq!(
            parse("", ""),
            Ok(Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                Position::fake()
            ))
        );
    }
}
