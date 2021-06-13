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
    use crate::{ast::*, types};

    // #[test]
    // fn parse_function_definition() {
    //     assert_eq!(
    //         parse("foo : Number -> Number -> Number\nfoo x y = 42", ""),
    //         Ok(Module::from_definitions(vec![FunctionDefinition::new(
    //             "foo",
    //             vec!["x".into(), "y".into()],
    //             Number::new(42.0, Position::dummy()),
    //             types::Function::new(
    //                 types::Number::new(Position::dummy()),
    //                 types::Function::new(
    //                     types::Number::new(Position::dummy()),
    //                     types::Number::new(Position::dummy()),
    //                     Position::dummy(),
    //                 ),
    //                 Position::dummy(),
    //             ),
    //             Position::dummy(),
    //         )
    //         .into()]))
    //     );
    // }

    // #[test]
    // fn parse_let_expression_with_single_definition() {
    //     assert_eq!(
    //         parse("x : Number\nx = (let x = 42\nin x)", ""),
    //         Ok(Module::from_definitions(vec![VariableDefinition::new(
    //             "x",
    //             Let::new(
    //                 vec![VariableDefinition::new(
    //                     "x",
    //                     Number::new(42.0, Position::dummy()),
    //                     types::Unknown::new(Position::dummy()),
    //                     Position::dummy(),
    //                 )
    //                 .into()],
    //                 Variable::new("x", Position::dummy()),
    //                 Position::dummy(),
    //             ),
    //             types::Number::new(Position::dummy()),
    //             Position::dummy(),
    //         )
    //         .into()]))
    //     );
    // }

    // #[test]
    // fn parse_let_expression_with_multiple_definitions() {
    //     assert_eq!(
    //         parse(
    //             indoc!(
    //                 "
    //                 main : Number -> Number
    //                     main x = (
    //                         let
    //                             f x = x
    //                             g x =
    //                             f x
    //                         in
    //                             g x
    //                     )
    //                     "
    //             ),
    //             ""
    //         ),
    //         Ok(Module::from_definitions(vec![FunctionDefinition::new(
    //             "main",
    //             vec!["x".into()],
    //             Let::new(
    //                 vec![
    //                     FunctionDefinition::new(
    //                         "f",
    //                         vec!["x".into()],
    //                         Variable::new("x", Position::dummy()),
    //                         types::Unknown::new(Position::dummy()),
    //                         Position::dummy(),
    //                     )
    //                     .into(),
    //                     FunctionDefinition::new(
    //                         "g",
    //                         vec!["x".into()],
    //                         Application::new(
    //                             Variable::new("f", Position::dummy()),
    //                             Variable::new("x", Position::dummy()),
    //                             Position::dummy(),
    //                         ),
    //                         types::Unknown::new(Position::dummy()),
    //                         Position::dummy(),
    //                     )
    //                     .into(),
    //                 ],
    //                 Application::new(
    //                     Variable::new("g", Position::dummy()),
    //                     Variable::new("x", Position::dummy()),
    //                     Position::dummy(),
    //                 ),
    //                 Position::dummy(),
    //             ),
    //             types::Function::new(
    //                 types::Number::new(Position::dummy()),
    //                 types::Number::new(Position::dummy()),
    //                 Position::dummy(),
    //             ),
    //             Position::dummy(),
    //         )
    //         .into()]))
    //     );
    // }

    // #[test]
    // fn parse_with_import_statement() {
    //     assert_eq!(
    //         parse(
    //             indoc!(
    //                 "
    //                 import Package.Module

    //                 main : Number -> Number
    //                 main x = x
    //                 "
    //             ),
    //             ""
    //         ),
    //         Ok(Module::new(
    //             Export::new(Default::default()),
    //             ExportForeign::new(Default::default()),
    //             vec![Import::new(ExternalModulePath::new(
    //                 "Package",
    //                 vec!["Module".into()]
    //             ))],
    //             vec![],
    //             vec![],
    //             vec![FunctionDefinition::new(
    //                 "main",
    //                 vec!["x".into()],
    //                 Variable::new("x", Position::dummy()),
    //                 types::Function::new(
    //                     types::Number::new(Position::dummy()),
    //                     types::Number::new(Position::dummy()),
    //                     Position::dummy()
    //                 ),
    //                 Position::dummy()
    //             )
    //             .into()]
    //         ))
    //     );
    // }

    // #[test]
    // fn parse_with_comment() {
    //     assert_eq!(
    //         parse(
    //             indoc!(
    //                 "
    //                 # foo is good
    //                 foo : Number -> Number
    //                 foo x = 42
    //                 "
    //             ),
    //             ""
    //         ),
    //         Ok(Module::from_definitions(vec![FunctionDefinition::new(
    //             "foo",
    //             vec!["x".into()],
    //             Number::new(42.0, Position::dummy()),
    //             types::Function::new(
    //                 types::Number::new(Position::dummy()),
    //                 types::Number::new(Position::dummy()),
    //                 Position::dummy()
    //             ),
    //             Position::dummy()
    //         )
    //         .into()]))
    //     );
    // }
}
