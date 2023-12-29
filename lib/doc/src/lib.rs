#![allow(unstable_name_collisions)]

mod ir;
mod markdown;

use ast::*;
use format::{format_function_signature, format_type_definition};
use indoc::indoc;
use ir::{build::*, *};
use itertools::Itertools;
use position::Position;
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Debug)]
pub struct Package {
    pub name: String,
    pub url: String,
    pub description: String,
    pub type_: String,
}

#[derive(Clone, Debug)]
pub struct Configuration {
    pub language: String,
    pub private_names: HashSet<String>,
}

#[derive(Clone, Debug)]
struct Context {
    comments: Vec<Comment>,
    configuration: Configuration,
}

pub fn generate(
    package: &Package,
    modules: &BTreeMap<ModulePath, (Module, Vec<Comment>)>,
    configuration: &Configuration,
) -> String {
    markdown::generate(&compile_package(package, modules, configuration))
}

fn compile_package(
    package: &Package,
    modules: &BTreeMap<ModulePath, (Module, Vec<Comment>)>,
    configuration: &Configuration,
) -> Section {
    section(
        text([code(&package.name), normal(" package")]),
        [
            text([normal(&package.description)]).into(),
            code_block(
                "json",
                format!(
                    indoc!(
                        "
                        {{
                          \"type\": \"{}\"
                        }}
                        "
                    ),
                    &package.type_,
                ),
            ),
        ],
        [section(
            text([normal("Install")]),
            [code_block(
                "json",
                format!(
                    indoc!(
                        "
                        {{
                          \"dependencies\": {{
                            \"{}\": \"{}\"
                          }} 
                        }}
                        "
                    ),
                    &package.name, &package.url,
                ),
            )],
            [],
        )]
        .into_iter()
        .chain(
            modules
                .iter()
                .filter(|(path, _)| ast::analysis::is_module_path_public(path))
                .map(|(path, (module, comments))| {
                    compile_module(
                        &Context {
                            comments: comments.to_vec(),
                            configuration: configuration.clone(),
                        },
                        path,
                        module,
                    )
                }),
        ),
    )
}

fn compile_module(context: &Context, path: &ModulePath, module: &Module) -> Section {
    section(
        text([code(path.to_string()), normal(" module")]),
        compile_first_block_comment(context, get_first_child_position(module)),
        [
            compile_type_definitions(context, module.type_definitions()),
            compile_function_definitions(context, module.function_definitions()),
        ],
    )
}

fn get_first_child_position(module: &Module) -> Option<&Position> {
    module
        .imports()
        .iter()
        .map(|import| import.position())
        .chain(
            module
                .foreign_imports()
                .iter()
                .map(|import| import.position()),
        )
        .chain(
            module
                .type_definitions()
                .iter()
                .map(|definition| definition.position()),
        )
        .chain(
            module
                .function_definitions()
                .iter()
                .map(|definition| definition.position()),
        )
        .next()
}

fn compile_type_definitions(context: &Context, definitions: &[TypeDefinition]) -> Section {
    let definitions = definitions
        .iter()
        .filter(|definition| {
            ast::analysis::is_name_public(definition.name())
                && !context
                    .configuration
                    .private_names
                    .contains(definition.name())
        })
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
                &context.configuration.language,
                if let TypeDefinition::RecordDefinition(record_definition) = definition {
                    if ast::analysis::is_record_open(record_definition) {
                        format_type_definition(definition)
                    } else {
                        format!("type {} {{\n  # ...\n}}", definition.name())
                    }
                } else {
                    format_type_definition(definition)
                },
            )]),
        [],
    )
}

fn compile_function_definitions(context: &Context, definitions: &[FunctionDefinition]) -> Section {
    let definitions = definitions
        .iter()
        .filter(|definition| {
            ast::analysis::is_name_public(definition.name())
                && !context
                    .configuration
                    .private_names
                    .contains(definition.name())
        })
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
            .map(|definition| compile_function_definition(context, definition)),
    )
}

fn compile_function_definition(context: &Context, definition: &FunctionDefinition) -> Section {
    section(
        text([code(definition.name())]),
        compile_last_block_comment(context, definition.position())
            .into_iter()
            .chain([code_block(
                &context.configuration.language,
                format_function_signature(definition.lambda()),
            )]),
        [],
    )
}

fn compile_first_block_comment(
    context: &Context,
    position: Option<&Position>,
) -> Option<Paragraph> {
    let comments = &context.comments;

    if comments
        .first()
        .map(|comment| comment.position().line_number())
        != Some(1)
    {
        return None;
    }

    let comments = &comments[..(1..comments.len())
        .find(|&index| {
            comments[index - 1].position().line_number() + 1
                != comments[index].position().line_number()
        })
        .unwrap_or(comments.len())];

    if comments
        .last()
        .map(|comment| comment.position().line_number() + 1)
        == position.map(|position| position.line_number())
    {
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

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use position::{test::PositionFake, Position};

    const TEST_LANGUAGE: &str = "pen";

    fn create_configuration() -> Configuration {
        Configuration {
            language: TEST_LANGUAGE.into(),
            private_names: Default::default(),
        }
    }

    fn create_context(comments: &[Comment]) -> Context {
        Context {
            comments: comments.to_vec(),
            configuration: create_configuration(),
        }
    }

    fn line_position(line_number: usize) -> Position {
        Position::new("", line_number, 1, "")
    }

    mod package {
        use super::*;
        use pretty_assertions::assert_eq;

        fn generate_package(
            package_name: &str,
            modules: &BTreeMap<ModulePath, (Module, Vec<Comment>)>,
        ) -> String {
            generate(
                &Package {
                    name: package_name.into(),
                    url: "https://foo.com/bar".into(),
                    description: "This package is cool.".into(),
                    type_: "application".into(),
                },
                modules,
                &create_configuration(),
            )
        }

        #[test]
        fn generate_empty() {
            assert_eq!(
                generate_package("Foo", &Default::default(),),
                indoc!(
                    "
                    # `Foo` package

                    This package is cool.

                    ```json
                    {
                      \"type\": \"application\"
                    }
                    ```

                    ## Install

                    ```json
                    {
                      \"dependencies\": {
                        \"Foo\": \"https://foo.com/bar\"
                      } 
                    }
                    ```
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

                    This package is cool.

                    ```json
                    {
                      \"type\": \"application\"
                    }
                    ```

                    ## Install

                    ```json
                    {
                      \"dependencies\": {
                        \"Foo\": \"https://foo.com/bar\"
                      } 
                    }
                    ```

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
        fn skip_private_module() {
            assert_eq!(
                generate_package(
                    "Foo",
                    &[(
                        ExternalModulePath::new("Foo", vec!["bar".into()]).into(),
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

                    This package is cool.

                    ```json
                    {
                      \"type\": \"application\"
                    }
                    ```

                    ## Install

                    ```json
                    {
                      \"dependencies\": {
                        \"Foo\": \"https://foo.com/bar\"
                      } 
                    }
                    ```
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

                    This package is cool.

                    ```json
                    {
                      \"type\": \"application\"
                    }
                    ```

                    ## Install

                    ```json
                    {
                      \"dependencies\": {
                        \"Foo\": \"https://foo.com/bar\"
                      } 
                    }
                    ```

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

        fn generate_with_private_names(
            path: &ModulePath,
            module: &Module,
            comments: &[Comment],
            private_names: &[&str],
        ) -> String {
            markdown::generate(&compile_module(
                &Context {
                    comments: comments.to_vec(),
                    configuration: Configuration {
                        language: TEST_LANGUAGE.into(),
                        private_names: private_names
                            .iter()
                            .map(|string| string.to_string())
                            .collect(),
                    },
                },
                path,
                module,
            ))
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
        fn generate_module_comment() {
            assert_eq!(
                generate(
                    &ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                    &Module::new(vec![], vec![], vec![], vec![], Position::fake()),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    # `Foo'Bar` module

                    foo

                    ## Types

                    No types are defined.

                    ## Functions

                    No functions are defined.
                    "
                )
            );
        }

        #[test]
        fn skip_module_comment_in_next_block() {
            assert_eq!(
                generate(
                    &ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                    &Module::new(vec![], vec![], vec![], vec![], Position::fake()),
                    &[
                        Comment::new("foo", line_position(1)),
                        Comment::new("bar", line_position(3))
                    ]
                ),
                indoc!(
                    "
                    # `Foo'Bar` module

                    foo

                    ## Types

                    No types are defined.

                    ## Functions

                    No functions are defined.
                    "
                )
            );
        }

        #[test]
        fn do_not_duplicate_definition_comment_as_module_comment() {
            assert_eq!(
                generate(
                    &ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                    &Module::new(
                        vec![],
                        vec![],
                        vec![RecordDefinition::new("Foo", vec![], line_position(2)).into()],
                        vec![],
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    # `Foo'Bar` module

                    ## Types

                    ### `Foo`

                    foo

                    ```pen
                    type Foo {}
                    ```

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
                        vec![FunctionDefinition::new(
                            "foo",
                            Lambda::new(
                                vec![],
                                types::Reference::new("none", Position::fake()),
                                Block::new(
                                    vec![],
                                    Variable::new("none", Position::fake()),
                                    Position::fake()
                                ),
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

        #[test]
        fn hide_hidden_type_definition() {
            assert_eq!(
                generate_with_private_names(
                    &ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                    &Module::new(
                        vec![],
                        vec![],
                        vec![RecordDefinition::new("Foo", vec![], Position::fake()).into()],
                        vec![],
                        Position::fake()
                    ),
                    &[],
                    &["Foo"]
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
        fn hide_hidden_function_definition() {
            assert_eq!(
                generate_with_private_names(
                    &ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                    &Module::new(
                        vec![],
                        vec![],
                        vec![],
                        vec![FunctionDefinition::new(
                            "Foo",
                            Lambda::new(
                                vec![],
                                types::Reference::new("none", Position::fake()),
                                Block::new(
                                    vec![],
                                    Variable::new("none", Position::fake()),
                                    Position::fake()
                                ),
                                Position::fake()
                            ),
                            None,
                            Position::fake()
                        )],
                        Position::fake()
                    ),
                    &[],
                    &["Foo"]
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
        use super::*;
        use ast::types::RecordField;

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
                        vec![RecordField::new(
                            "Bar",
                            types::Reference::new("none", Position::fake()),
                            Position::fake()
                        )],
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
                        vec![RecordField::new(
                            "bar",
                            types::Reference::new("none", Position::fake()),
                            Position::fake()
                        )],
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
                      # ...
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
                    &TypeAlias::new(
                        "Foo",
                        types::Reference::new("none", Position::fake()),
                        Position::fake()
                    )
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

        fn generate(definition: &FunctionDefinition, comments: &[Comment]) -> String {
            markdown::generate(&compile_function_definition(
                &create_context(comments),
                definition,
            ))
        }

        #[test]
        fn generate_definition() {
            assert_eq!(
                generate(
                    &FunctionDefinition::new(
                        "Foo",
                        Lambda::new(
                            vec![],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        None,
                        Position::fake()
                    ),
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
