mod ir;
mod markdown;

use ast::*;
use ir::{build::*, *};

pub fn generate(path: &ModulePath, module: &Module, comments: &[Comment]) -> String {
    markdown::generate(&compile_module(path, module, comments))
}

fn compile_module(path: &ModulePath, module: &Module, comments: &[Comment]) -> Section {
    section(
        text([code(format!("{}", path))]),
        [],
        [
            compile_type_definitions(module.type_definitions(), comments),
            compile_definitions(module.definitions(), comments),
        ],
    )
}

fn compile_type_definitions(definitions: &[TypeDefinition], comments: &[Comment]) -> Section {
    section(
        text([normal("Types")]),
        if definitions.is_empty() {
            vec![text([normal("No types are defined.")]).into()]
        } else {
            vec![]
        },
        definitions
            .iter()
            .map(|definition| compile_type_definition(definition, comments)),
    )
}

fn compile_type_definition(definition: &TypeDefinition, comments: &[Comment]) -> Section {
    match definition {
        TypeDefinition::RecordDefinition(definition) => {
            section(text([code(definition.name())]), [], [])
        }
        TypeDefinition::TypeAlias(alias) => section(text([code(alias.name())]), [], []),
    }
}

fn compile_definitions(definitions: &[Definition], comments: &[Comment]) -> Section {
    section(
        text([normal("Functions")]),
        if definitions.is_empty() {
            vec![text([normal("No functions are defined.")]).into()]
        } else {
            vec![]
        },
        [],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use position::{test::PositionFake, Position};

    mod module {
        use super::*;

        fn generate(path: &ModulePath, module: &Module, comments: &[Comment]) -> String {
            markdown::generate(&compile_module(path, module, comments))
        }

        #[test]
        fn generate_empty() {
            assert_eq!(
                generate(
                    &ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                    &Module::new(vec![], vec![], vec![], vec![], Position::fake()),
                    &[]
                ),
                indoc!(
                    "
                    # `Foo'Bar`

                    ## Types

                    No types are defined.

                    ## Functions

                    No functions are defined.
                    "
                )
            );
        }
    }

    mod type_definition {
        use super::*;

        fn generate(definition: &TypeDefinition, comments: &[Comment]) -> String {
            markdown::generate(&compile_type_definition(definition, comments))
        }

        #[test]
        fn generate_empty() {
            assert_eq!(
                generate(
                    &RecordDefinition::new("Foo", vec![], Position::fake()).into(),
                    &[]
                ),
                indoc!(
                    "
                    # `Foo`
                    "
                )
            );
        }
    }
}
