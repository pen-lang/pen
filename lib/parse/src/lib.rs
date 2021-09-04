mod error;
mod parsers;
mod utilities;

use ast::ast;
use combine::Parser;
pub use error::ParseError;
use parsers::{module, stream};

pub fn parse(source: &str, path: &str) -> Result<ast::Module, ParseError> {
    module()
        .parse(stream(source, path))
        .map(|(module, _)| module)
        .map_err(|error| ParseError::new(source, path, error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
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
