#![allow(unstable_name_collisions)]

mod ir;
mod markdown;

use ast::*;
use format::{format_function_signature, format_type_definition};
use ir::{build::*, *};
use itertools::Itertools;
use position::Position;

struct Context {
    comments: Vec<Comment>,
    language: String,
}

pub fn generate(
    path: &ModulePath,
    module: &Module,
    comments: &[Comment],
    language: &str,
) -> String {
    markdown::generate(&compile_module(
        &Context {
            comments: comments.to_vec(),
            language: language.into(),
        },
        path,
        module,
    ))
}

fn compile_module(context: &Context, path: &ModulePath, module: &Module) -> Section {
    section(
        text([code(path.to_string())]),
        [],
        [
            compile_type_definitions(context, module.type_definitions()),
            compile_definitions(context, module.definitions()),
        ],
    )
}

fn compile_type_definitions(context: &Context, definitions: &[TypeDefinition]) -> Section {
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
        compile_block_comment(context, definition.position())
            .into_iter()
            .chain([code_block(
                &context.language,
                format_type_definition(definition),
            )]),
        [],
    )
}

fn compile_definitions(context: &Context, definitions: &[Definition]) -> Section {
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
        compile_block_comment(context, definition.position())
            .into_iter()
            .chain([code_block(
                &context.language,
                format_function_signature(definition.lambda()),
            )]),
        [],
    )
}

fn compile_block_comment(context: &Context, position: &Position) -> Option<Paragraph> {
    let comments = &context.comments[..context
        .comments
        .partition_point(|comment| comment.position().line_number() < position.line_number())];

    if comments.is_empty() {
        None
    } else {
        Some(
            text(
                comments
                    .iter()
                    .map(|comment| normal(comment.line().trim_end()))
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

    fn create_context(comments: &[Comment]) -> Context {
        Context {
            comments: comments.to_vec(),
            language: "pen".into(),
        }
    }

    fn line_position(line_number: usize) -> Position {
        Position::new("", line_number, 1, "")
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
            markdown::generate(&compile_type_definition(
                &create_context(comments),
                definition,
            ))
        }

        #[test]
        fn generate_record_definition() {
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
    }
}
