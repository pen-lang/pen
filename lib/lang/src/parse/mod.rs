#[macro_use]
mod attempt;
mod error;
mod parsers;
mod utilities;

use crate::ast;
use combine::Parser;
pub use error::ParseError;
use parsers::{module, stream};

pub fn parse(source_content: &str, source_name: &str) -> Result<ast::UnresolvedModule, ParseError> {
    module()
        .parse(stream(source_content, source_name))
        .map(|(module, _)| module)
        .map_err(|error| ParseError::new(source_name, &error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ast::*, debug::SourceInformation, path::*, types};
    use indoc::indoc;

    #[test]
    fn parse_function_definition() {
        assert_eq!(
            parse("foo : Number -> Number -> Number\nfoo x y = 42", ""),
            Ok(UnresolvedModule::from_definitions(vec![
                FunctionDefinition::new(
                    "foo",
                    vec!["x".into(), "y".into()],
                    Number::new(42.0, SourceInformation::dummy()),
                    types::Function::new(
                        types::Number::new(SourceInformation::dummy()),
                        types::Function::new(
                            types::Number::new(SourceInformation::dummy()),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    ),
                    SourceInformation::dummy(),
                )
                .into()
            ]))
        );
    }

    #[test]
    fn parse_let_expression_with_single_definition() {
        assert_eq!(
            parse("x : Number\nx = (let x = 42\nin x)", ""),
            Ok(UnresolvedModule::from_definitions(vec![
                VariableDefinition::new(
                    "x",
                    Let::new(
                        vec![VariableDefinition::new(
                            "x",
                            Number::new(42.0, SourceInformation::dummy()),
                            types::Unknown::new(SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        )
                        .into()],
                        Variable::new("x", SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    ),
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy(),
                )
                .into()
            ]))
        );
    }

    #[test]
    fn parse_let_expression_with_multiple_definitions() {
        assert_eq!(
            parse(
                indoc!(
                    "
                    main : Number -> Number
                        main x = (
                            let
                                f x = x
                                g x =
                                f x
                            in
                                g x
                        )
                        "
                ),
                ""
            ),
            Ok(UnresolvedModule::from_definitions(vec![
                FunctionDefinition::new(
                    "main",
                    vec!["x".into()],
                    Let::new(
                        vec![
                            FunctionDefinition::new(
                                "f",
                                vec!["x".into()],
                                Variable::new("x", SourceInformation::dummy()),
                                types::Unknown::new(SourceInformation::dummy()),
                                SourceInformation::dummy(),
                            )
                            .into(),
                            FunctionDefinition::new(
                                "g",
                                vec!["x".into()],
                                Application::new(
                                    Variable::new("f", SourceInformation::dummy()),
                                    Variable::new("x", SourceInformation::dummy()),
                                    SourceInformation::dummy(),
                                ),
                                types::Unknown::new(SourceInformation::dummy()),
                                SourceInformation::dummy(),
                            )
                            .into(),
                        ],
                        Application::new(
                            Variable::new("g", SourceInformation::dummy()),
                            Variable::new("x", SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    ),
                    types::Function::new(
                        types::Number::new(SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    ),
                    SourceInformation::dummy(),
                )
                .into()
            ]))
        );
    }

    #[test]
    fn parse_with_import_statement() {
        assert_eq!(
            parse(
                indoc!(
                    "
                    import Package.Module

                    main : Number -> Number
                    main x = x
                    "
                ),
                ""
            ),
            Ok(UnresolvedModule::new(
                Export::new(Default::default()),
                ExportForeign::new(Default::default()),
                vec![UnresolvedImport::new(ExternalUnresolvedModulePath::new(
                    "Package",
                    vec!["Module".into()]
                ))],
                vec![],
                vec![],
                vec![FunctionDefinition::new(
                    "main",
                    vec!["x".into()],
                    Variable::new("x", SourceInformation::dummy()),
                    types::Function::new(
                        types::Number::new(SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    SourceInformation::dummy()
                )
                .into()]
            ))
        );
    }

    #[test]
    fn parse_with_comment() {
        assert_eq!(
            parse(
                indoc!(
                    "
                    # foo is good
                    foo : Number -> Number
                    foo x = 42
                    "
                ),
                ""
            ),
            Ok(UnresolvedModule::from_definitions(vec![
                FunctionDefinition::new(
                    "foo",
                    vec!["x".into()],
                    Number::new(42.0, SourceInformation::dummy()),
                    types::Function::new(
                        types::Number::new(SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    SourceInformation::dummy()
                )
                .into()
            ]))
        );
    }
}
