mod comment;
mod error;
mod operations;
mod parsers;
mod stream;

use combine::Parser;
use comment::Comment;
pub use error::ParseError;
use parsers::{comments, module};
use stream::stream;

pub fn parse(source: &str, path: &str) -> Result<ast::Module, ParseError> {
    module()
        .parse(stream(source, path))
        .map(|(module, _)| module)
        .map_err(|error| ParseError::new(source, path, error))
}

pub fn parse_comments(source: &str, path: &str) -> Result<Vec<Comment>, ParseError> {
    comments()
        .parse(stream(source, path))
        .map(|(module, _)| module)
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
