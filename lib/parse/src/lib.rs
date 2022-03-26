mod comment;
mod error;
mod operations;
mod parsers;
mod stream;

use combine::Parser;
use comment::Comment;
pub use error::ParseError;
use parsers::module;
use stream::stream;

pub fn parse(source: &str, path: &str) -> Result<ast::Module, ParseError> {
    module()
        .parse(stream(source, path))
        .map(|(module, _)| module)
        .map_err(|error| ParseError::new(source, path, error))
}

pub fn parse_comments(_source: &str, _path: &str) -> Result<Vec<Comment>, ParseError> {
    Ok(vec![])
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
