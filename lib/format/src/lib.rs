use ast::{types::Type, *};

const INDENT: &str = "  ";

// TODO Merge comments.
pub fn format(module: &Module) -> String {
    [
        module
            .imports()
            .iter()
            .map(format_import)
            .collect::<Vec<_>>()
            .join("\n"),
        module
            .foreign_imports()
            .iter()
            .map(format_foreign_import)
            .collect::<Vec<_>>()
            .join("\n"),
    ]
    .into_iter()
    .filter(|string| !string.is_empty())
    .chain(module.type_definitions().iter().map(format_type_definition))
    .chain(module.definitions().iter().map(format_definition))
    .collect::<Vec<_>>()
    .join("\n\n")
        + "\n"
}

fn format_import(import: &Import) -> String {
    ["import".into(), format_module_path(import.module_path())]
        .into_iter()
        .chain(import.prefix().map(|prefix| format!("as {}", prefix)))
        .chain(if import.unqualified_names().is_empty() {
            None
        } else {
            Some(format!("{{ {} }}", import.unqualified_names().join(", ")))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_module_path(path: &ModulePath) -> String {
    match path {
        ModulePath::External(path) => {
            format!(
                "{}'{}",
                path.package(),
                format_module_path_components(path.components())
            )
        }
        ModulePath::Internal(path) => {
            format!("'{}", format_module_path_components(path.components()))
        }
    }
}

fn format_module_path_components(components: &[String]) -> String {
    components.join("'")
}

fn format_foreign_import(import: &ForeignImport) -> String {
    ["import foreign".into()]
        .into_iter()
        .chain(match import.calling_convention() {
            CallingConvention::C => Some("\"c\"".into()),
            CallingConvention::Native => None,
        })
        .chain([import.name().into(), format_type(import.type_())])
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_type_definition(definition: &TypeDefinition) -> String {
    match definition {
        TypeDefinition::RecordDefinition(definition) => format_record_definition(definition),
        TypeDefinition::TypeAlias(alias) => format_type_alias(alias),
    }
}

fn format_record_definition(definition: &RecordDefinition) -> String {
    let head = ["type", definition.name(), "{"].join(" ");

    if definition.fields().is_empty() {
        head + "}"
    } else {
        [
            head,
            definition
                .fields()
                .iter()
                .map(|field| {
                    indent_line(field.name().to_owned() + " " + &format_type(field.type_()))
                })
                .collect::<Vec<_>>()
                .join(",\n"),
            "}".into(),
        ]
        .join("\n")
    }
}

fn format_type_alias(alias: &TypeAlias) -> String {
    [
        "type".into(),
        alias.name().into(),
        "=".into(),
        format_type(alias.type_()),
    ]
    .join(" ")
}

fn format_definition(definition: &Definition) -> String {
    definition
        .foreign_export()
        .map(|export| {
            ["export"]
                .into_iter()
                .chain(match export.calling_convention() {
                    CallingConvention::C => Some("\"c\""),
                    CallingConvention::Native => None,
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .into_iter()
        .chain([
            definition.name().into(),
            "=".into(),
            format_lambda(definition.lambda()),
        ])
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_type(type_: &Type) -> String {
    match type_ {
        Type::Any(_) => "any".into(),
        Type::Boolean(_) => "boolean".into(),
        Type::Function(function) => format!(
            "\\({}) {}",
            function
                .arguments()
                .iter()
                .map(format_type)
                .collect::<Vec<_>>()
                .join(", "),
            format_type(function.result()),
        ),
        Type::List(list) => format!("[{}]", format_type(list.element())),
        Type::Map(map) => format!(
            "{{{}: {}}}",
            format_type(map.key()),
            format_type(map.value())
        ),
        Type::None(_) => "none".into(),
        Type::Number(_) => "number".into(),
        Type::Record(record) => record.name().into(),
        Type::Reference(reference) => reference.name().into(),
        Type::String(_) => "string".into(),
        Type::Union(union) => format!(
            "{} | {}",
            {
                let type_ = format_type(union.lhs());

                if union.lhs().is_function() {
                    format!("({})", type_)
                } else {
                    type_
                }
            },
            format_type(union.rhs())
        ),
    }
}

fn format_lambda(lambda: &Lambda) -> String {
    format!(
        "\\({}) {} {}",
        lambda
            .arguments()
            .iter()
            .map(|argument| format!("{} {}", argument.name(), format_type(argument.type_())))
            .collect::<Vec<_>>()
            .join(", "),
        format_type(lambda.result_type()),
        if lambda.arguments().is_empty() && lambda.body().statements().is_empty() {
            format!("{{ {} }}", format_expression(lambda.body().expression()))
        } else {
            format_block(lambda.body())
        }
    )
}

fn format_block(block: &Block) -> String {
    ["{".into()]
        .into_iter()
        .chain(
            block
                .statements()
                .iter()
                .map(format_statement)
                .map(indent_line),
        )
        .chain([indent_line(format_expression(block.expression()))])
        .chain(["}".into()])
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_statement(statement: &Statement) -> String {
    statement
        .name()
        .map(|name| format!("{} = ", name))
        .into_iter()
        .chain([format_expression(statement.expression())])
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_expression(expression: &Expression) -> String {
    match expression {
        Expression::None(_) => "none".into(),
        Expression::Number(number) => format!("{}", number.value()),
        _ => todo!(),
    }
}

fn indent_line(string: impl AsRef<str>) -> String {
    INDENT.to_owned() + string.as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::{test::PositionFake, Position};

    #[test]
    fn format_empty_module() {
        assert_eq!(
            format(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            "\n"
        );
    }

    #[test]
    fn format_internal_module_import() {
        assert_eq!(
            format(&Module::new(
                vec![Import::new(
                    InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                    None,
                    vec![],
                )],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            "import 'Foo'Bar\n"
        );
    }

    #[test]
    fn format_external_module_import() {
        assert_eq!(
            format(&Module::new(
                vec![Import::new(
                    ExternalModulePath::new("Package", vec!["Foo".into(), "Bar".into()]),
                    None,
                    vec![]
                )],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            "import Package'Foo'Bar\n"
        );
    }

    #[test]
    fn format_prefixed_module_import() {
        assert_eq!(
            format(&Module::new(
                vec![Import::new(
                    InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                    Some("Baz".into()),
                    vec![]
                )],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            "import 'Foo'Bar as Baz\n"
        );
    }

    #[test]
    fn format_unqualified_module_import() {
        assert_eq!(
            format(&Module::new(
                vec![Import::new(
                    InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                    None,
                    vec!["Baz".into(), "Blah".into()]
                )],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            "import 'Foo'Bar { Baz, Blah }\n"
        );
    }

    #[test]
    fn format_foreign_import() {
        assert_eq!(
            format(&Module::new(
                vec![],
                vec![ForeignImport::new(
                    "foo",
                    CallingConvention::Native,
                    types::Function::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                )],
                vec![],
                vec![],
                Position::fake()
            )),
            "import foreign foo \\() none\n"
        );
    }

    #[test]
    fn format_foreign_import_with_c_calling_convention() {
        assert_eq!(
            format(&Module::new(
                vec![],
                vec![ForeignImport::new(
                    "foo",
                    CallingConvention::C,
                    types::Function::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                )],
                vec![],
                vec![],
                Position::fake()
            )),
            "import foreign \"c\" foo \\() none\n"
        );
    }

    #[test]
    fn format_function_type_in_union_type() {
        assert_eq!(
            format_type(
                &types::Union::new(
                    types::Function::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Position::fake()
                    ),
                    types::None::new(Position::fake()),
                    Position::fake()
                )
                .into()
            ),
            "(\\() none) | none"
        );
    }

    #[test]
    fn format_record_definition_with_no_field() {
        assert_eq!(
            format(&Module::new(
                vec![],
                vec![],
                vec![RecordDefinition::new("foo", vec![], Position::fake()).into()],
                vec![],
                Position::fake()
            )),
            "type foo {}\n"
        );
    }

    #[test]
    fn format_record_definition_with_field() {
        assert_eq!(
            format(&Module::new(
                vec![],
                vec![],
                vec![RecordDefinition::new(
                    "foo",
                    vec![types::RecordField::new(
                        "foo",
                        types::None::new(Position::fake())
                    )],
                    Position::fake()
                )
                .into()],
                vec![],
                Position::fake()
            )),
            "type foo {\n  foo none\n}\n"
        );
    }

    #[test]
    fn format_type_alias() {
        assert_eq!(
            format(&Module::new(
                vec![],
                vec![],
                vec![
                    TypeAlias::new("foo", types::None::new(Position::fake()), Position::fake())
                        .into()
                ],
                vec![],
                Position::fake()
            )),
            "type foo = none\n"
        );
    }

    #[test]
    fn format_definition_with_no_argument_and_no_statement() {
        assert_eq!(
            format(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "foo",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Block::new(vec![], None::new(Position::fake()), Position::fake()),
                        Position::fake(),
                    ),
                    None,
                    Position::fake()
                )],
                Position::fake()
            )),
            "foo = \\() none { none }\n"
        );
    }

    #[test]
    fn format_definition_with_argument() {
        assert_eq!(
            format(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::fake()))],
                        types::None::new(Position::fake()),
                        Block::new(vec![], None::new(Position::fake()), Position::fake()),
                        Position::fake(),
                    ),
                    None,
                    Position::fake()
                )],
                Position::fake()
            )),
            "foo = \\(x none) none {\n  none\n}\n"
        );
    }

    #[test]
    fn format_definition_with_statement() {
        assert_eq!(
            format(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "foo",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Block::new(
                            vec![Statement::new(
                                None,
                                None::new(Position::fake()),
                                Position::fake()
                            )],
                            None::new(Position::fake()),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    None,
                    Position::fake()
                )],
                Position::fake()
            )),
            "foo = \\() none {\n  none\n  none\n}\n"
        );
    }
}
