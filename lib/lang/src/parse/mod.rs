#[macro_use]
mod attempt;
mod error;
mod parsers;
mod utilities;

use crate::ast;
use combine::Parser;
pub use error::ParseError;
use parsers::{module, stream};

pub fn parse(source_content: &str, path: &str) -> Result<ast::Module, ParseError> {
    module()
        .parse(stream(source_content, path))
        .map(|(module, _)| module)
        .map_err(|error| ParseError::new(path, &error))
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
                vec![],
                Position::fake()
            ))
        );
    }
}
