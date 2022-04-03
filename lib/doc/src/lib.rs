#![allow(unstable_name_collisions)]

mod ir;
mod markdown;

use ast::*;
use format::{format_function_signature, format_type_definition};
use ir::{build::*, *};
use itertools::Itertools;
use position::Position;
use std::collections::BTreeMap;

struct Context {
    comments: Vec<Comment>,
    language: String,
}

// TODO Generate package description in some way. Maybe from README.md?
pub fn generate(
    package_name: &str,
    modules: &BTreeMap<ModulePath, (Module, Vec<Comment>)>,
    language: &str,
) -> String {
    markdown::generate(&compile_package(package_name, modules, language))
}

fn compile_package(
    package_name: &str,
    modules: &BTreeMap<ModulePath, (Module, Vec<Comment>)>,
    language: &str,
) -> Section {
    section(
        text([code(package_name), normal(" package")]),
        [],
        modules.iter().map(|(path, (module, comments))| {
            compile_module(
                &Context {
                    comments: comments.to_vec(),
                    language: language.into(),
                },
                path,
                module,
            )
        }),
    )
}

fn compile_module(context: &Context, path: &ModulePath, module: &Module) -> Section {
    section(
        text([code(path.to_string()), normal(" module")]),
        [],
        [
            compile_type_definitions(context, module.type_definitions()),
            compile_definitions(context, module.definitions()),
        ],
    )
}

fn compile_type_definitions(context: &Context, definitions: &[TypeDefinition]) -> Section {
    let definitions = definitions
        .iter()
        .filter(|definition| ast::analysis::is_name_public(definition.name()))
        .collect::<Vec<_>>();

    section(
        text([normal("Types")]),
        if definitions.is_empty() {
            Some(text([normal("No types are defined.")]).into())
        } else {
            None
        },
        definitions
            .iter()
            .map(|definition| compile_type_definition(context, definition)),
    )
}

fn compile_type_definition(context: &Context, definition: &TypeDefinition) -> Section {
    section(
        text([code(definition.name())]),
        compile_last_block_comment(context, definition.position())
            .into_iter()
            .chain([code_block(
                &context.language,
                if let TypeDefinition::RecordDefinition(record_definition) = definition {
                    if ast::analysis::is_record_open(record_definition) {
                        format_type_definition(definition)
                    } else {
                        format!("type {} {{\n  ...\n}}", definition.name())
                    }
                } else {
                    format_type_definition(definition)
                },
            )]),
        [],
    )
}

fn compile_definitions(context: &Context, definitions: &[Definition]) -> Section {
    let definitions = definitions
        .iter()
        .filter(|definition| ast::analysis::is_name_public(definition.name()))
        .collect::<Vec<_>>();

    section(
        text([normal("Functions")]),
        if definitions.is_empty() {
            Some(text([normal("No functions are defined.")]).into())
        } else {
            None
        },
        definitions
            .iter()
            .map(|definition| compile_definition(context, definition)),
    )
}

fn compile_definition(context: &Context, definition: &Definition) -> Section {
    section(
        text([code(definition.name())]),
        compile_last_block_comment(context, definition.position())
            .into_iter()
            .chain([code_block(
                &context.language,
                format_function_signature(definition.lambda()),
            )]),
        [],
    )
}

fn compile_last_block_comment(context: &Context, position: &Position) -> Option<Paragraph> {
    let end = context
        .comments
        .iter()
        .rposition(|comment| comment.position().line_number() == position.line_number() - 1)?;
    let comments = &context.comments[..end + 1];
    let comments = &comments[(1..comments.len())
        .rfind(|&index| {
            comments[index - 1].position().line_number()
                != comments[index].position().line_number() - 1
        })
        .unwrap_or_default()..];

    if comments.is_empty() {
        None
    } else {
        Some(
            text(
                comments
                    .iter()
                    .map(|comment| normal(comment.line().trim()))
                    .intersperse(normal("\n"))
                    .collect::<Vec<_>>(),
            )
            .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use position::{test::PositionFake, Position};

    const TEST_LANGUAGE: &str = "pen";

    fn create_context(comments: &[Comment]) -> Context {
        Context {
            comments: comments.to_vec(),
            language: TEST_LANGUAGE.into(),
        }
    }

    fn line_position(line_number: usize) -> Position {
        Position::new("", line_number, 1, "")
    }

    mod package {
        use super::*;

        fn generate_package(
            package_name: &str,
            modules: &BTreeMap<ModulePath, (Module, Vec<Comment>)>,
        ) -> String {
            generate(package_name, modules, TEST_LANGUAGE)
        }

        #[test]
        fn generate_empty() {
            assert_eq!(
                generate_package("Foo", &Default::default(),),
                indoc!(
                    "
                    # `Foo` package
                    "
                )
            );
        }

        #[test]
        fn generate_module() {
            assert_eq!(
                generate_package(
                    "Foo",
                    &[(
                        ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                        (
                            Module::new(vec![], vec![], vec![], vec![], Position::fake()),
                            Default::default()
                        )
                    )]
                    .into_iter()
                    .collect(),
                ),
                indoc!(
                    "
                    # `Foo` package

                    ## `Foo'Bar` module

                    ### Types

                    No types are defined.

                    ### Functions

                    No functions are defined.
                    "
                )
            );
        }

        #[test]
        fn generate_modules() {
            assert_eq!(
                generate_package(
                    "Foo",
                    &[
                        (
                            ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                            (
                                Module::new(vec![], vec![], vec![], vec![], Position::fake()),
                                Default::default()
                            )
                        ),
                        (
                            ExternalModulePath::new("Foo", vec!["Baz".into()]).into(),
                            (
                                Module::new(vec![], vec![], vec![], vec![], Position::fake()),
                                Default::default()
                            )
                        )
                    ]
                    .into_iter()
                    .collect(),
                ),
                indoc!(
                    "
                    # `Foo` package

                    ## `Foo'Bar` module

                    ### Types

                    No types are defined.

                    ### Functions

                    No functions are defined.

                    ## `Foo'Baz` module

                    ### Types

                    No types are defined.

                    ### Functions

                    No functions are defined.
                    "
                )
            );
        }
    }

    mod module {
        use super::*;

        fn generate(path: &ModulePath, module: &Module, comments: &[Comment]) -> String {
            markdown::generate(&compile_module(&create_context(comments), path, module))
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
                    # `Foo'Bar` module

                    ## Types

                    No types are defined.

                    ## Functions

                    No functions are defined.
                    "
                )
            );
        }

        #[test]
        fn hide_private_type_definition() {
            assert_eq!(
                generate(
                    &ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                    &Module::new(
                        vec![],
                        vec![],
                        vec![RecordDefinition::new("foo", vec![], Position::fake()).into()],
                        vec![],
                        Position::fake()
                    ),
                    &[]
                ),
                indoc!(
                    "
                    # `Foo'Bar` module

                    ## Types

                    No types are defined.

                    ## Functions

                    No functions are defined.
                    "
                )
            );
        }

        #[test]
        fn hide_private_function_definition() {
            assert_eq!(
                generate(
                    &ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                    &Module::new(
                        vec![],
                        vec![],
                        vec![],
                        vec![Definition::new(
                            "foo",
                            Lambda::new(
                                vec![],
                                types::None::new(Position::fake()),
                                Block::new(vec![], None::new(Position::fake()), Position::fake()),
                                Position::fake()
                            ),
                            None,
                            Position::fake()
                        )],
                        Position::fake()
                    ),
                    &[]
                ),
                indoc!(
                    "
                    # `Foo'Bar` module

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
        use ast::types::RecordField;

        use super::*;

        fn generate(definition: &TypeDefinition, comments: &[Comment]) -> String {
            markdown::generate(&compile_type_definition(
                &create_context(comments),
                definition,
            ))
        }

        #[test]
        fn generate_empty_record_definition() {
            assert_eq!(
                generate(
                    &RecordDefinition::new("Foo", vec![], Position::fake()).into(),
                    &[]
                ),
                indoc!(
                    "
                    # `Foo`

                    ```pen
                    type Foo {}
                    ```
                    "
                )
            );
        }

        #[test]
        fn generate_open_record_definition() {
            assert_eq!(
                generate(
                    &RecordDefinition::new(
                        "Foo",
                        vec![RecordField::new("Bar", types::None::new(Position::fake()))],
                        Position::fake()
                    )
                    .into(),
                    &[]
                ),
                indoc!(
                    "
                    # `Foo`

                    ```pen
                    type Foo {
                      Bar none
                    }
                    ```
                    "
                )
            );
        }

        #[test]
        fn generate_closed_record_definition() {
            assert_eq!(
                generate(
                    &RecordDefinition::new(
                        "Foo",
                        vec![RecordField::new("bar", types::None::new(Position::fake()))],
                        Position::fake()
                    )
                    .into(),
                    &[]
                ),
                indoc!(
                    "
                    # `Foo`

                    ```pen
                    type Foo {
                      ...
                    }
                    ```
                    "
                )
            );
        }

        #[test]
        fn generate_type_alias() {
            assert_eq!(
                generate(
                    &TypeAlias::new("Foo", types::None::new(Position::fake()), Position::fake())
                        .into(),
                    &[]
                ),
                indoc!(
                    "
                    # `Foo`

                    ```pen
                    type Foo = none
                    ```
                    "
                )
            );
        }
    }

    mod definition {
        use super::*;

        fn generate(definition: &Definition, comments: &[Comment]) -> String {
            markdown::generate(&compile_definition(&create_context(comments), definition))
        }

        #[test]
        fn generate_definition() {
            assert_eq!(
                generate(
                    &Definition::new(
                        "Foo",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
                            Position::fake()
                        ),
                        None,
                        Position::fake()
                    )
                    .into(),
                    &[]
                ),
                indoc!(
                    "
                    # `Foo`

                    ```pen
                    \\() none
                    ```
                    "
                )
            );
        }
    }

    mod comment {
        use super::*;

        fn generate(definition: &TypeDefinition, comments: &[Comment]) -> String {
            markdown::generate(&compile_type_definition(
                &create_context(comments),
                definition,
            ))
        }

        #[test]
        fn generate_comment() {
            assert_eq!(
                generate(
                    &RecordDefinition::new("Foo", vec![], line_position(2)).into(),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    # `Foo`

                    foo

                    ```pen
                    type Foo {}
                    ```
                    "
                )
            );
        }

        #[test]
        fn trim_preceding_spaces() {
            assert_eq!(
                generate(
                    &RecordDefinition::new("Foo", vec![], line_position(2)).into(),
                    &[Comment::new(" foo", line_position(1))]
                ),
                indoc!(
                    "
                    # `Foo`

                    foo

                    ```pen
                    type Foo {}
                    ```
                    "
                )
            );
        }

        #[test]
        fn trim_trailing_spaces() {
            assert_eq!(
                generate(
                    &RecordDefinition::new("Foo", vec![], line_position(2)).into(),
                    &[Comment::new("foo ", line_position(1))]
                ),
                indoc!(
                    "
                    # `Foo`

                    foo

                    ```pen
                    type Foo {}
                    ```
                    "
                )
            );
        }

        #[test]
        fn generate_comment_of_multiple_lines() {
            assert_eq!(
                generate(
                    &RecordDefinition::new("Foo", vec![], line_position(3)).into(),
                    &[
                        Comment::new("foo", line_position(1)),
                        Comment::new("bar", line_position(2))
                    ]
                ),
                indoc!(
                    "
                    # `Foo`

                    foo
                    bar

                    ```pen
                    type Foo {}
                    ```
                    "
                )
            );
        }

        #[test]
        fn generate_comment_of_multiple_paragraphs() {
            assert_eq!(
                generate(
                    &RecordDefinition::new("Foo", vec![], line_position(4)).into(),
                    &[
                        Comment::new("foo", line_position(1)),
                        Comment::new("", line_position(2)),
                        Comment::new("bar", line_position(3))
                    ]
                ),
                indoc!(
                    "
                    # `Foo`

                    foo

                    bar

                    ```pen
                    type Foo {}
                    ```
                    "
                )
            );
        }

        #[test]
        fn skip_comment_of_previous_block() {
            assert_eq!(
                generate(
                    &RecordDefinition::new("Foo", vec![], line_position(3)).into(),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    # `Foo`

                    ```pen
                    type Foo {}
                    ```
                    "
                )
            );
        }

        #[test]
        fn generate_comment_after_skipped_comment() {
            assert_eq!(
                generate(
                    &RecordDefinition::new("Foo", vec![], line_position(4)).into(),
                    &[
                        Comment::new("foo", line_position(1)),
                        Comment::new("bar", line_position(3))
                    ]
                ),
                indoc!(
                    "
                    # `Foo`

                    bar

                    ```pen
                    type Foo {}
                    ```
                    "
                )
            );
        }
    }
}
