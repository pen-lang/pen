#![allow(unstable_name_collisions)]

mod context;

use ast::{analysis::operator_priority, types::Type, *};
use context::Context;
use itertools::Itertools;
use mfmt::{
    empty, flatten, flatten_if, indent, line, line_suffix, r#break, sequence,
    utility::{count_lines, is_broken},
    Document,
};
use position::Position;

pub fn format(module: &Module, comments: &[Comment]) -> String {
    let mut context = Context::new(comments);
    let context = &mut context;

    let (external_imports, internal_imports) = module
        .imports()
        .iter()
        .partition::<Vec<_>, _>(|import| matches!(import.module_path(), ModulePath::External(_)));

    mfmt::format(
        &[
            compile_imports(context, &external_imports),
            compile_imports(context, &internal_imports),
            compile_foreign_imports(context, module.foreign_imports()),
            sequence(
                module
                    .type_definitions()
                    .iter()
                    .map(|definition| compile_type_definition(context, definition))
                    .intersperse(line()),
            ),
            sequence(
                module
                    .function_definitions()
                    .iter()
                    .map(|definition| compile_function_definition(context, definition))
                    .intersperse(line()),
            ),
            compile_remaining_block_comment(context),
        ]
        .into_iter()
        .fold(empty(), |all, document| {
            if count_lines(&document) == 0 {
                all
            } else {
                sequence([
                    if count_lines(&all) == 0 {
                        empty()
                    } else {
                        sequence([all, line()])
                    },
                    document,
                ])
            }
        }),
    )
}

pub fn format_type_definition(definition: &TypeDefinition) -> String {
    mfmt::format(&compile_type_definition(&mut Context::new(&[]), definition))
}

pub fn format_function_signature(lambda: &Lambda) -> String {
    mfmt::format(&compile_signature(
        &mut Context::new(&[]),
        lambda.arguments(),
        lambda.result_type(),
        lambda.position(),
    ))
}

fn compile_imports(context: &mut Context, imports: &[&Import]) -> Document {
    sequence(
        imports
            .iter()
            .sorted_by_key(|import| import.module_path())
            .map(|import| compile_import(context, import)),
    )
}

fn compile_import(context: &mut Context, import: &Import) -> Document {
    sequence([
        compile_block_comment(context, import.position()),
        "import ".into(),
        compile_module_path(import.module_path()),
        if let Some(prefix) = import.prefix() {
            sequence([" as ", prefix])
        } else {
            empty()
        },
        if import.unqualified_names().is_empty() {
            empty()
        } else {
            sequence([
                " { ".into(),
                sequence(
                    import
                        .unqualified_names()
                        .iter()
                        .map(|name| name.name())
                        .sorted()
                        .intersperse(", "),
                ),
                " }".into(),
            ])
        },
        line(),
    ])
}

fn compile_module_path(path: &ModulePath) -> Document {
    match path {
        ModulePath::External(path) => sequence([
            path.package().into(),
            "'".into(),
            compile_module_path_components(path.components()),
        ]),
        ModulePath::Internal(path) => sequence([
            "'".into(),
            compile_module_path_components(path.components()),
        ]),
    }
}

fn compile_module_path_components(components: &[String]) -> Document {
    components.join("'").into()
}

fn compile_foreign_imports(context: &mut Context, imports: &[ForeignImport]) -> Document {
    sequence(
        imports
            .iter()
            .map(|import| compile_foreign_import(context, import)),
    )
}

fn compile_foreign_import(context: &mut Context, import: &ForeignImport) -> Document {
    sequence([
        compile_block_comment(context, import.position()),
        "import foreign".into(),
        match import.calling_convention() {
            CallingConvention::C => " \"c\"".into(),
            CallingConvention::Native => empty(),
        },
        " ".into(),
        import.name().into(),
        " ".into(),
        compile_type(import.type_()),
        line(),
    ])
}

fn compile_type_definition(context: &mut Context, definition: &TypeDefinition) -> Document {
    match definition {
        TypeDefinition::RecordDefinition(definition) => {
            compile_record_definition(context, definition)
        }
        TypeDefinition::TypeAlias(alias) => compile_type_alias(context, alias),
    }
}

fn compile_record_definition(context: &mut Context, definition: &RecordDefinition) -> Document {
    sequence([
        compile_block_comment(context, definition.position()),
        "type ".into(),
        definition.name().into(),
        " {".into(),
        if definition.fields().is_empty() {
            empty()
        } else {
            sequence([
                indent(sequence(definition.fields().iter().map(|field| {
                    sequence([
                        line(),
                        compile_line_comment(context, field.position(), |_| {
                            sequence([field.name().into(), " ".into(), compile_type(field.type_())])
                        }),
                    ])
                }))),
                line(),
            ])
        },
        "}".into(),
        line(),
    ])
}

fn compile_type_alias(context: &mut Context, alias: &TypeAlias) -> Document {
    let type_ = compile_type(alias.type_());

    sequence([
        compile_block_comment(context, alias.position()),
        "type ".into(),
        alias.name().into(),
        " =".into(),
        if is_broken(&type_) {
            indent(sequence([line(), type_]))
        } else {
            sequence([" ".into(), type_])
        },
        line(),
    ])
}

fn compile_function_definition(context: &mut Context, definition: &FunctionDefinition) -> Document {
    sequence([
        compile_block_comment(context, definition.position()),
        if let Some(export) = definition.foreign_export() {
            sequence([
                "foreign ".into(),
                match export.calling_convention() {
                    CallingConvention::C => "\"c\" ".into(),
                    CallingConvention::Native => empty(),
                },
            ])
        } else {
            empty()
        },
        definition.name().into(),
        " = ".into(),
        compile_lambda(context, definition.lambda()),
        line(),
    ])
}

fn compile_type(type_: &Type) -> Document {
    match type_ {
        Type::Function(function) => sequence([
            "\\(".into(),
            sequence(
                function
                    .arguments()
                    .iter()
                    .map(compile_type)
                    .intersperse(", ".into()),
            ),
            ") ".into(),
            compile_type(function.result()),
        ]),
        Type::List(list) => sequence(["[".into(), compile_type(list.element()), "]".into()]),
        Type::Map(map) => sequence([
            "{".into(),
            compile_type(map.key()),
            ": ".into(),
            compile_type(map.value()),
            "}".into(),
        ]),
        Type::Record(record) => record.name().into(),
        Type::Reference(reference) => reference.name().into(),
        Type::Union(_) => {
            let types = collect_union_types(type_);

            let union = sequence(
                types
                    .iter()
                    .enumerate()
                    .map(|(index, type_)| {
                        let document = compile_type(type_);

                        if index != types.len() - 1 && matches!(type_, Type::Function(_)) {
                            sequence(["(".into(), document, ")".into()])
                        } else {
                            document
                        }
                    })
                    .intersperse(sequence([" |".into(), line()])),
            );

            if types.len() == 1
                || types.get(0).map(|type_| type_.position().line_number())
                    == types.get(1).map(|type_| type_.position().line_number())
                    && !is_broken(&union)
            {
                flatten(union)
            } else {
                r#break(union)
            }
        }
    }
}

fn compile_lambda(context: &mut Context, lambda: &Lambda) -> Document {
    sequence([
        compile_signature(
            context,
            lambda.arguments(),
            lambda.result_type(),
            lambda.position(),
        ),
        " ".into(),
        flatten_if(
            are_arguments_flat(lambda.arguments(), lambda.position())
                && lambda.position().line_number()
                    == lambda.body().expression().position().line_number(),
            compile_block(context, lambda.body()),
        ),
    ])
}

fn compile_signature(
    context: &mut Context,
    arguments: &[Argument],
    result_type: &Type,
    position: &Position,
) -> Document {
    let flat = are_arguments_flat(arguments, position);
    let separator = sequence([",".into(), line()]);

    let arguments = sequence(
        arguments
            .iter()
            .map(|argument| {
                compile_line_comment(context, argument.position(), |_| {
                    sequence([
                        argument.name().into(),
                        " ".into(),
                        compile_type(argument.type_()),
                    ])
                })
            })
            .intersperse(separator.clone()),
    );

    sequence([
        "\\(".into(),
        if flat {
            flatten(arguments)
        } else {
            r#break(sequence([indent(sequence([line(), arguments])), separator]))
        },
        ") ".into(),
        compile_type(result_type),
    ])
}

fn are_arguments_flat(arguments: &[Argument], position: &Position) -> bool {
    arguments.is_empty()
        || Some(position.line_number())
            == arguments
                .get(0)
                .map(|argument| argument.position().line_number())
}

fn compile_block(context: &mut Context, block: &Block) -> Document {
    let statements = sequence(
        block
            .statements()
            .iter()
            .zip(
                block
                    .statements()
                    .iter()
                    .skip(1)
                    .map(|statement| statement.position())
                    .chain([block.expression().position()]),
            )
            .map(|(statement, next_position)| {
                let block_comment = compile_block_comment(context, statement.position());
                // TODO Use end positions of spans when they are available.
                let line_count = next_position.line_number() as isize
                    - statement.position().line_number() as isize;
                let statement_document = compile_statement(context, statement);

                let extra_line = if (count_lines(&statement_document)
                    + context
                        .peek_comments_before(next_position.line_number())
                        .count()) as isize
                    >= line_count
                {
                    empty()
                } else {
                    line()
                };

                sequence([block_comment, statement_document, extra_line])
            }),
    );

    sequence([
        "{".into(),
        indent(sequence([
            line(),
            statements,
            compile_line_comment(context, block.expression().position(), |context| {
                compile_expression(context, block.expression())
            }),
        ])),
        line(),
        "}".into(),
    ])
}

fn compile_statement(context: &mut Context, statement: &Statement) -> Document {
    sequence([
        if let Some(name) = statement.name() {
            sequence([name, " = "])
        } else {
            empty()
        },
        compile_expression(context, statement.expression()),
        compile_suffix_comment(context, statement.position()),
        r#break(line()),
    ])
}

fn compile_expression(context: &mut Context, expression: &Expression) -> Document {
    match expression {
        Expression::BinaryOperation(operation) => compile_binary_operation(context, operation),
        Expression::Call(call) => {
            let separator = sequence([",".into(), line()]);
            let function = compile_expression(context, call.function());
            let arguments = sequence(
                call.arguments()
                    .iter()
                    .map(|argument| {
                        compile_line_comment(context, argument.position(), |context| {
                            compile_expression(context, argument)
                        })
                    })
                    .intersperse(separator.clone()),
            );

            sequence([
                function,
                "(".into(),
                if call.arguments().is_empty()
                    || Some(call.function().position().line_number())
                        == call
                            .arguments()
                            .get(0)
                            .map(|expression| expression.position().line_number())
                        && !is_broken(&arguments)
                {
                    flatten(arguments)
                } else {
                    r#break(sequence([indent(sequence([line(), arguments])), separator]))
                },
                ")".into(),
            ])
        }
        Expression::If(if_) => compile_if(context, if_),
        Expression::IfList(if_) => sequence([
            sequence(["if [", if_.first_name(), ", ...", if_.rest_name(), "] = "]),
            compile_expression(context, if_.list()),
            " ".into(),
            compile_block(context, if_.then()),
            " else ".into(),
            compile_block(context, if_.else_()),
        ]),
        Expression::IfMap(if_) => sequence([
            sequence(["if ", if_.name(), " = "]),
            compile_expression(context, if_.map()),
            "[".into(),
            compile_expression(context, if_.key()),
            "] ".into(),
            compile_block(context, if_.then()),
            " else ".into(),
            compile_block(context, if_.else_()),
        ]),
        Expression::IfType(if_) => compile_if_type(context, if_),
        Expression::Lambda(lambda) => compile_lambda(context, lambda),
        Expression::List(list) => compile_list(context, list),
        Expression::ListComprehension(comprehension) => {
            let elements = sequence([
                line(),
                compile_line_comment(context, comprehension.element().position(), |context| {
                    compile_expression(context, comprehension.element())
                }),
                line(),
                sequence(
                    comprehension
                        .branches()
                        .iter()
                        .map(|branch| {
                            compile_line_comment(context, branch.position(), |context| {
                                sequence(
                                    ["for ".into()]
                                        .into_iter()
                                        .chain(
                                            branch
                                                .names()
                                                .iter()
                                                .map(|string| string.as_str().into())
                                                .intersperse(", ".into()),
                                        )
                                        .chain([" in ".into()])
                                        .chain(
                                            branch
                                                .iteratees()
                                                .iter()
                                                .map(|iteratee| {
                                                    compile_expression(context, iteratee)
                                                })
                                                .intersperse(", ".into())
                                                .collect::<Vec<_>>(),
                                        )
                                        .chain(branch.condition().map(|condition| {
                                            sequence([
                                                " if ".into(),
                                                compile_expression(context, condition),
                                            ])
                                        })),
                                )
                            })
                        })
                        .intersperse(line()),
                ),
            ]);

            sequence([
                "[".into(),
                compile_type(comprehension.type_()),
                if comprehension.position().line_number()
                    == comprehension.element().position().line_number()
                    && !is_broken(&elements)
                {
                    flatten(elements)
                } else {
                    r#break(sequence([indent(elements), line()]))
                },
                "]".into(),
            ])
        }
        Expression::Map(map) => compile_map(context, map),
        Expression::Number(number) => match number.value() {
            NumberRepresentation::Binary(string) => "0b".to_owned() + string,
            NumberRepresentation::Hexadecimal(string) => "0x".to_owned() + &string.to_uppercase(),
            NumberRepresentation::FloatingPoint(string) => string.clone(),
        }
        .into(),
        Expression::Record(record) => {
            let separator = sequence([",".into(), line()]);
            let elements = sequence(
                record
                    .record()
                    .map(|record| {
                        compile_line_comment(context, record.position(), |context| {
                            sequence(["...".into(), compile_expression(context, record)])
                        })
                    })
                    .into_iter()
                    .chain(record.fields().iter().map(|field| {
                        compile_line_comment(context, field.position(), |context| {
                            sequence([
                                field.name().into(),
                                ": ".into(),
                                compile_expression(context, field.expression()),
                            ])
                        })
                    }))
                    .intersperse(separator.clone()),
            );

            sequence([
                record.type_name().into(),
                "{".into(),
                if record.record().is_none() && record.fields().is_empty()
                    || Some(record.position().line_number())
                        == if let Some(record) = record.record() {
                            Some(record.position())
                        } else {
                            record.fields().get(0).map(|field| field.position())
                        }
                        .map(|position| position.line_number())
                        && !is_broken(&elements)
                {
                    flatten(elements)
                } else {
                    r#break(sequence([indent(sequence([line(), elements])), separator]))
                },
                "}".into(),
            ])
        }
        Expression::RecordDeconstruction(deconstruction) => sequence([
            compile_expression(context, deconstruction.expression()),
            ".".into(),
            deconstruction.name().into(),
        ]),
        Expression::String(string) => sequence(["\"", string.value(), "\""]),
        Expression::UnaryOperation(operation) => {
            let operand = compile_expression(context, operation.expression());
            let operand = if matches!(operation.expression(), Expression::BinaryOperation(_)) {
                sequence(["(".into(), operand, ")".into()])
            } else {
                operand
            };

            match operation.operator() {
                UnaryOperator::Not => sequence(["!".into(), operand]),
                UnaryOperator::Try => sequence([operand, "?".into()]),
            }
        }
        Expression::Variable(variable) => variable.name().into(),
    }
}

fn compile_if(context: &mut Context, if_: &If) -> Document {
    let document = sequence([
        sequence(if_.branches().iter().map(|branch| {
            sequence([
                "if ".into(),
                compile_expression(context, branch.condition()),
                " ".into(),
                compile_block(context, branch.block()),
                " else ".into(),
            ])
        })),
        compile_block(context, if_.else_()),
    ]);

    flatten_if(
        if_.branches().len() == 1
            && Some(if_.position().line_number())
                == if_
                    .branches()
                    .get(0)
                    .map(|branch| branch.block().expression().position().line_number()),
        document,
    )
}

fn compile_if_type(context: &mut Context, if_: &IfType) -> Document {
    let document = sequence([
        "if ".into(),
        if_.name().into(),
        " = ".into(),
        compile_expression(context, if_.argument()),
        " as ".into(),
        sequence(
            if_.branches()
                .iter()
                .map(|branch| {
                    sequence([
                        compile_type(branch.type_()),
                        " ".into(),
                        compile_block(context, branch.block()),
                    ])
                })
                .intersperse(" else if ".into()),
        ),
        if let Some(block) = if_.else_() {
            sequence([" else ".into(), compile_block(context, block)])
        } else {
            empty()
        },
    ]);

    flatten_if(
        if_.branches().len() + if_.else_().iter().count() <= 2
            && Some(if_.position().line_number())
                == if_
                    .branches()
                    .get(0)
                    .map(|branch| branch.block().expression().position().line_number()),
        document,
    )
}

fn compile_list(context: &mut Context, list: &List) -> Document {
    let separator = Document::from(",");
    let elements = sequence(
        list.elements()
            .iter()
            .map(|element| {
                sequence([
                    line(),
                    compile_line_comment(context, element.position(), |context| match element {
                        ListElement::Multiple(expression) => {
                            sequence(["...".into(), compile_expression(context, expression)])
                        }
                        ListElement::Single(expression) => compile_expression(context, expression),
                    }),
                ])
            })
            .intersperse(separator.clone()),
    );

    sequence([
        "[".into(),
        compile_type(list.type_()),
        if list.elements().is_empty()
            || Some(list.position().line_number())
                == list
                    .elements()
                    .get(0)
                    .map(|element| element.position().line_number())
                && !is_broken(&elements)
        {
            flatten(elements)
        } else {
            r#break(sequence([indent(elements), separator, line()]))
        },
        "]".into(),
    ])
}

fn compile_map(context: &mut Context, map: &Map) -> Document {
    let type_ = sequence([
        compile_type(map.key_type()),
        ": ".into(),
        compile_type(map.value_type()),
    ]);
    let separator = Document::from(",");
    let elements = sequence(
        map.elements()
            .iter()
            .map(|element| {
                sequence([
                    line(),
                    compile_line_comment(context, element.position(), |context| match element {
                        MapElement::Multiple(expression) => {
                            sequence(["...".into(), compile_expression(context, expression)])
                        }
                        MapElement::Single(entry) => sequence([
                            compile_expression(context, entry.key()),
                            ": ".into(),
                            compile_expression(context, entry.value()),
                        ]),
                    }),
                ])
            })
            .intersperse(separator.clone()),
    );

    sequence([
        "{".into(),
        type_,
        if map.elements().is_empty()
            || Some(map.position().line_number())
                == map
                    .elements()
                    .get(0)
                    .map(|element| element.position().line_number())
                && !is_broken(&elements)
        {
            flatten(elements)
        } else {
            r#break(sequence([indent(elements), separator, line()]))
        },
        "}".into(),
    ])
}

fn compile_binary_operation(context: &mut Context, operation: &BinaryOperation) -> Document {
    let document = sequence([
        compile_operand(context, operation.lhs(), operation.operator()),
        indent(sequence([
            line(),
            compile_binary_operator(operation.operator()),
            " ".into(),
            compile_operand(context, operation.rhs(), operation.operator()),
        ])),
    ]);

    flatten_if(
        operation.lhs().position().line_number() == operation.rhs().position().line_number(),
        document,
    )
}

fn compile_operand(
    context: &mut Context,
    operand: &Expression,
    parent_operator: BinaryOperator,
) -> Document {
    let document = compile_expression(context, operand);

    if match operand {
        Expression::BinaryOperation(operation) => Some(operation),
        _ => None,
    }
    .map(|operand| operator_priority(operand.operator()) < operator_priority(parent_operator))
    .unwrap_or_default()
    {
        sequence(["(".into(), document, ")".into()])
    } else {
        document
    }
}

fn compile_binary_operator(operator: BinaryOperator) -> Document {
    match operator {
        BinaryOperator::Or => "|",
        BinaryOperator::And => "&",
        BinaryOperator::Equal => "==",
        BinaryOperator::NotEqual => "!=",
        BinaryOperator::LessThan => "<",
        BinaryOperator::LessThanOrEqual => "<=",
        BinaryOperator::GreaterThan => ">",
        BinaryOperator::GreaterThanOrEqual => ">=",
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
    }
    .into()
}

fn compile_line_comment(
    context: &mut Context,
    position: &Position,
    document: impl Fn(&mut Context) -> Document,
) -> Document {
    sequence([
        compile_block_comment(context, position),
        document(context),
        compile_suffix_comment(context, position),
    ])
}

fn compile_suffix_comment(context: &mut Context, position: &Position) -> Document {
    sequence(
        context
            .drain_current_comment(position.line_number())
            .map(|comment| line_suffix(" #".to_owned() + comment.line().trim_end())),
    )
}

fn compile_block_comment(context: &mut Context, position: &Position) -> Document {
    compile_all_comments(
        &context
            .drain_comments_before(position.line_number())
            .collect::<Vec<_>>(),
        Some(position.line_number()),
    )
}

fn compile_remaining_block_comment(context: &mut Context) -> Document {
    compile_all_comments(
        &context
            .drain_comments_before(usize::MAX)
            .collect::<Vec<_>>(),
        None,
    )
}

fn compile_all_comments(comments: &[Comment], last_line_number: Option<usize>) -> Document {
    sequence(
        comments
            .iter()
            .zip(
                comments
                    .iter()
                    .skip(1)
                    .map(|comment| comment.position().line_number())
                    .chain([last_line_number.unwrap_or(0)]),
            )
            .map(|(comment, next_line_number)| {
                sequence([
                    "#".into(),
                    comment.line().trim_end().into(),
                    r#break(line()),
                    if comment.position().line_number() + 1 < next_line_number {
                        line()
                    } else {
                        empty()
                    },
                ])
            }),
    )
}

fn collect_union_types(type_: &Type) -> Vec<Type> {
    match type_ {
        Type::Union(union) => collect_union_types(union.lhs())
            .into_iter()
            .chain(collect_union_types(union.rhs()))
            .collect(),
        _ => vec![type_.clone()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn line_position(line: usize) -> Position {
        Position::new("", line, 1, "")
    }

    fn format_module(module: &Module) -> String {
        format(module, &[])
    }

    #[test]
    fn format_empty_module() {
        assert_eq!(
            format_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            ""
        );
    }

    mod import {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn format_internal_module_import() {
            assert_eq!(
                format_module(&Module::new(
                    vec![Import::new(
                        InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                        None,
                        vec![],
                        Position::fake(),
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
                format_module(&Module::new(
                    vec![Import::new(
                        ExternalModulePath::new("Package", vec!["Foo".into(), "Bar".into()]),
                        None,
                        vec![],
                        Position::fake()
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
                format_module(&Module::new(
                    vec![Import::new(
                        InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                        Some("Baz".into()),
                        vec![],
                        Position::fake()
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
        fn format_import_with_unqualified_names() {
            assert_eq!(
                format_module(&Module::new(
                    vec![Import::new(
                        InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                        None,
                        vec![
                            UnqualifiedName::new("Baz", Position::fake()),
                            UnqualifiedName::new("Blah", Position::fake())
                        ],
                        Position::fake()
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
        fn format_import_with_unsorted_unqualified_names() {
            assert_eq!(
                format_module(&Module::new(
                    vec![Import::new(
                        InternalModulePath::new(vec!["Foo".into()]),
                        None,
                        vec![
                            UnqualifiedName::new("B", Position::fake()),
                            UnqualifiedName::new("A", Position::fake()),
                        ],
                        line_position(2),
                    )],
                    vec![],
                    vec![],
                    vec![],
                    Position::fake()
                )),
                "import 'Foo { A, B }\n"
            );
        }

        #[test]
        fn sort_module_imports_with_external_paths() {
            assert_eq!(
                format_module(&Module::new(
                    vec![
                        Import::new(
                            ExternalModulePath::new("Foo", vec!["Foo".into()]),
                            None,
                            vec![],
                            Position::fake(),
                        ),
                        Import::new(
                            ExternalModulePath::new("Bar", vec!["Bar".into()]),
                            None,
                            vec![],
                            Position::fake()
                        )
                    ],
                    vec![],
                    vec![],
                    vec![],
                    Position::fake()
                )),
                indoc!(
                    "
                    import Bar'Bar
                    import Foo'Foo
                    "
                )
            );
        }

        #[test]
        fn sort_module_imports_with_internal_paths() {
            assert_eq!(
                format_module(&Module::new(
                    vec![
                        Import::new(
                            InternalModulePath::new(vec!["Foo".into()]),
                            None,
                            vec![],
                            Position::fake(),
                        ),
                        Import::new(
                            InternalModulePath::new(vec!["Bar".into()]),
                            None,
                            vec![],
                            Position::fake()
                        )
                    ],
                    vec![],
                    vec![],
                    vec![],
                    Position::fake()
                )),
                indoc!(
                    "
                    import 'Bar
                    import 'Foo
                    "
                )
            );
        }

        #[test]
        fn sort_module_imports_with_external_and_internal_paths() {
            assert_eq!(
                format_module(&Module::new(
                    vec![
                        Import::new(
                            InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                            None,
                            vec![],
                            Position::fake(),
                        ),
                        Import::new(
                            ExternalModulePath::new("Package", vec!["Foo".into(), "Bar".into()]),
                            None,
                            vec![],
                            Position::fake()
                        )
                    ],
                    vec![],
                    vec![],
                    vec![],
                    Position::fake()
                )),
                indoc!(
                    "
                    import Package'Foo'Bar

                    import 'Foo'Bar
                    "
                )
            );
        }
    }

    #[test]
    fn format_foreign_import() {
        assert_eq!(
            format_module(&Module::new(
                vec![],
                vec![ForeignImport::new(
                    "foo",
                    CallingConvention::Native,
                    types::Function::new(
                        vec![],
                        types::Reference::new("none", Position::fake()),
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
            format_module(&Module::new(
                vec![],
                vec![ForeignImport::new(
                    "foo",
                    CallingConvention::C,
                    types::Function::new(
                        vec![],
                        types::Reference::new("none", Position::fake()),
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
    fn format_record_definition_with_no_field() {
        assert_eq!(
            format_module(&Module::new(
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
            format_module(&Module::new(
                vec![],
                vec![],
                vec![RecordDefinition::new(
                    "foo",
                    vec![types::RecordField::new(
                        "foo",
                        types::Reference::new("none", Position::fake()),
                        Position::fake()
                    )],
                    Position::fake()
                )
                .into()],
                vec![],
                Position::fake()
            )),
            indoc!(
                "
                type foo {
                  foo none
                }
                "
            )
        );
    }

    #[test]
    fn format_record_definition_with_two_fields() {
        assert_eq!(
            format_module(&Module::new(
                vec![],
                vec![],
                vec![RecordDefinition::new(
                    "foo",
                    vec![
                        types::RecordField::new(
                            "foo",
                            types::Reference::new("none", Position::fake()),
                            Position::fake()
                        ),
                        types::RecordField::new(
                            "bar",
                            types::Reference::new("none", Position::fake()),
                            Position::fake()
                        )
                    ],
                    Position::fake()
                )
                .into()],
                vec![],
                Position::fake()
            )),
            indoc!(
                "
                type foo {
                  foo none
                  bar none
                }
                "
            )
        );
    }

    mod type_alias {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn format_type_alias() {
            assert_eq!(
                format_module(&Module::new(
                    vec![],
                    vec![],
                    vec![TypeAlias::new(
                        "foo",
                        types::Reference::new("none", Position::fake()),
                        Position::fake()
                    )
                    .into()],
                    vec![],
                    Position::fake()
                )),
                "type foo = none\n"
            );
        }

        #[test]
        fn format_multiple_type_aliases() {
            assert_eq!(
                format_module(&Module::new(
                    vec![],
                    vec![],
                    vec![
                        TypeAlias::new(
                            "foo",
                            types::Reference::new("none", Position::fake()),
                            Position::fake()
                        )
                        .into(),
                        TypeAlias::new(
                            "bar",
                            types::Reference::new("none", Position::fake()),
                            Position::fake()
                        )
                        .into()
                    ],
                    vec![],
                    Position::fake()
                )),
                indoc!(
                    "
                type foo = none

                type bar = none
                "
                ),
            );
        }

        #[test]
        fn format_type_alias_with_broken_type() {
            assert_eq!(
                format_module(&Module::new(
                    vec![],
                    vec![],
                    vec![TypeAlias::new(
                        "foo",
                        types::Union::new(
                            types::Reference::new("number", line_position(1)),
                            types::Reference::new("none", line_position(2)),
                            Position::fake()
                        ),
                        Position::fake(),
                    )
                    .into()],
                    vec![],
                    Position::fake()
                )),
                "type foo =\n  number |\n  none\n"
            );
        }
    }

    mod type_ {
        use super::*;
        use pretty_assertions::assert_eq;

        fn format_type(type_: &Type) -> String {
            mfmt::format(&compile_type(type_))
        }

        #[test]
        fn format_function_type_in_union_type() {
            assert_eq!(
                format_type(
                    &types::Union::new(
                        types::Function::new(
                            vec![],
                            types::Reference::new("none", Position::fake()),
                            Position::fake()
                        ),
                        types::Reference::new("none", Position::fake()),
                        Position::fake()
                    )
                    .into()
                ),
                "(\\() none) | none"
            );
        }

        #[test]
        fn format_function_multi_line_union_type() {
            assert_eq!(
                format_type(
                    &types::Union::new(
                        types::Reference::new("number", line_position(1)),
                        types::Reference::new("none", line_position(2)),
                        Position::fake()
                    )
                    .into()
                ),
                "number |\nnone"
            );
        }

        #[test]
        fn format_function_multi_line_union_type_with_3_types() {
            assert_eq!(
                format_type(
                    &types::Union::new(
                        types::Reference::new("number", line_position(1)),
                        types::Union::new(
                            types::Reference::new("string", line_position(2)),
                            types::Reference::new("none", line_position(2)),
                            Position::fake()
                        ),
                        Position::fake()
                    )
                    .into()
                ),
                "number |\nstring |\nnone"
            );
        }
    }

    mod definition {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn format_with_no_argument_and_no_statement() {
            assert_eq!(
                format_module(&Module::new(
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
        fn format_multiple() {
            let definition = FunctionDefinition::new(
                "foo",
                Lambda::new(
                    vec![],
                    types::Reference::new("none", Position::fake()),
                    Block::new(
                        vec![],
                        Variable::new("none", Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                None,
                Position::fake(),
            );

            assert_eq!(
                format_module(&Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![definition.clone(), definition],
                    Position::fake()
                )),
                indoc!(
                    "
                    foo = \\() none { none }

                    foo = \\() none { none }
                    "
                ),
            );
        }

        #[test]
        fn format_with_argument() {
            assert_eq!(
                format_module(&Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![FunctionDefinition::new(
                        "foo",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Reference::new("none", Position::fake()),
                                Position::fake()
                            )],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        None,
                        Position::fake()
                    )],
                    Position::fake()
                )),
                "foo = \\(x none) none { none }\n"
            );
        }

        #[test]
        fn format_with_statement() {
            assert_eq!(
                format_module(&Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![FunctionDefinition::new(
                        "foo",
                        Lambda::new(
                            vec![],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![Statement::new(
                                    None,
                                    Variable::new("none", Position::fake()),
                                    Position::fake()
                                )],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        None,
                        Position::fake()
                    )],
                    Position::fake()
                )),
                indoc!(
                    "
                foo = \\() none {
                  none
                  none
                }
                "
                )
            );
        }

        #[test]
        fn format_returning_lambda() {
            assert_eq!(
                format_module(&Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![FunctionDefinition::new(
                        "foo",
                        Lambda::new(
                            vec![],
                            types::Function::new(
                                vec![],
                                types::Reference::new("none", Position::fake()),
                                Position::fake()
                            ),
                            Block::new(
                                vec![],
                                Lambda::new(
                                    vec![],
                                    types::Reference::new("none", Position::fake()),
                                    Block::new(
                                        vec![],
                                        Variable::new("none", Position::fake()),
                                        Position::fake()
                                    ),
                                    Position::fake(),
                                ),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        None,
                        Position::fake()
                    )],
                    Position::fake()
                )),
                "foo = \\() \\() none { \\() none { none } }\n"
            );
        }

        #[test]
        fn format_with_foreign_export() {
            assert_eq!(
                format_module(&Module::new(
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
                            Position::fake(),
                        ),
                        Some(ForeignExport::new(CallingConvention::Native)),
                        Position::fake()
                    )],
                    Position::fake()
                )),
                "foreign foo = \\() none { none }\n"
            );
        }

        #[test]
        fn format_with_foreign_export_and_custom_calling_convention() {
            assert_eq!(
                format_module(&Module::new(
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
                            Position::fake(),
                        ),
                        Some(ForeignExport::new(CallingConvention::C)),
                        Position::fake()
                    )],
                    Position::fake()
                )),
                "foreign \"c\" foo = \\() none { none }\n"
            );
        }
    }

    mod block {
        use super::*;
        use pretty_assertions::assert_eq;

        fn format(block: &Block) -> String {
            mfmt::format(&compile_block(&mut Context::new(&[]), block)) + "\n"
        }

        fn format_with_comments(block: &Block, comments: &[Comment]) -> String {
            mfmt::format(&compile_block(&mut Context::new(comments), block)) + "\n"
        }

        #[test]
        fn format_() {
            assert_eq!(
                format(&Block::new(
                    vec![],
                    Variable::new("none", Position::fake()),
                    Position::fake()
                )),
                indoc!(
                    "
                    {
                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_statement() {
            assert_eq!(
                format(&Block::new(
                    vec![Statement::new(
                        None,
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    Variable::new("none", Position::fake()),
                    Position::fake()
                )),
                indoc!(
                    "
                    {
                      f()
                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_statement_with_name() {
            assert_eq!(
                format(&Block::new(
                    vec![Statement::new(
                        Some("x".into()),
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    Variable::new("none", Position::fake()),
                    Position::fake()
                )),
                indoc!(
                    "
                    {
                      x = f()
                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_statement_with_no_blank_line() {
            assert_eq!(
                format(&Block::new(
                    vec![Statement::new(
                        None,
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        line_position(1)
                    )],
                    Variable::new("none", line_position(2)),
                    Position::fake()
                )),
                indoc!(
                    "
                    {
                      f()
                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_statement_with_one_blank_line() {
            assert_eq!(
                format(&Block::new(
                    vec![Statement::new(
                        None,
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        line_position(1)
                    )],
                    Variable::new("none", line_position(3)),
                    Position::fake()
                )),
                indoc!(
                    "
                    {
                      f()

                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_statement_with_two_blank_lines() {
            assert_eq!(
                format(&Block::new(
                    vec![Statement::new(
                        None,
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        line_position(1)
                    )],
                    Variable::new("none", line_position(4)),
                    Position::fake()
                )),
                indoc!(
                    "
                    {
                      f()

                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_statement_with_trimmed_blank_line() {
            assert_eq!(
                format_module(&Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![FunctionDefinition::new(
                        "foo",
                        Lambda::new(
                            vec![],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![Statement::new(
                                    None,
                                    Call::new(
                                        Variable::new("f", Position::fake()),
                                        vec![],
                                        Position::fake()
                                    ),
                                    line_position(1)
                                )],
                                Variable::new("none", line_position(3)),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        None,
                        Position::fake()
                    )],
                    Position::fake()
                )),
                indoc!(
                    "
                    foo = \\() none {
                      f()

                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_block_comment_for_statement() {
            assert_eq!(
                format_with_comments(
                    &Block::new(
                        vec![Statement::new(
                            Some("x".into()),
                            Variable::new("none", Position::fake()),
                            line_position(2)
                        )],
                        Variable::new("none", line_position(3)),
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    {
                      #foo
                      x = none
                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_result_expression_in_block() {
            assert_eq!(
                format_with_comments(
                    &Block::new(
                        vec![],
                        Variable::new("none", line_position(2)),
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    {
                      #foo
                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_suffix_comment_of_last_expression() {
            assert_eq!(
                format_with_comments(
                    &Block::new(
                        vec![],
                        Variable::new("none", line_position(2)),
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(2))]
                ),
                indoc!(
                    "
                    {
                      none #foo
                    }
                    "
                )
            );
        }

        #[test]
        fn format_comment_between_statement_and_expression_in_block() {
            assert_eq!(
                format_with_comments(
                    &Block::new(
                        vec![Statement::new(
                            Some("x".into()),
                            Variable::new("none", Position::fake()),
                            line_position(1)
                        )],
                        Variable::new("none", line_position(3)),
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(2))]
                ),
                indoc!(
                    "
                    {
                      x = none
                      #foo
                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_suffix_comment_after_statement() {
            assert_eq!(
                format_with_comments(
                    &Block::new(
                        vec![Statement::new(
                            Some("x".into()),
                            Variable::new("none", Position::fake()),
                            line_position(2)
                        )],
                        Variable::new("none", line_position(3)),
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(2))]
                ),
                indoc!(
                    "
                    {
                      x = none #foo
                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_space_between_two_statements_with_comment_in_first_statement() {
            assert_eq!(
                format_with_comments(
                    &Block::new(
                        vec![
                            Statement::new(
                                Some("x".into()),
                                If::new(
                                    vec![IfBranch::new(
                                        Variable::new("true", Position::fake()),
                                        Block::new(
                                            vec![],
                                            Variable::new("none", line_position(4)),
                                            Position::fake()
                                        )
                                    )],
                                    Block::new(
                                        vec![],
                                        Variable::new("none", Position::fake()),
                                        Position::fake()
                                    ),
                                    Position::fake()
                                ),
                                line_position(2)
                            ),
                            Statement::new(
                                Some("y".into()),
                                Variable::new("none", Position::fake()),
                                line_position(9)
                            )
                        ],
                        Variable::new("none", line_position(10)),
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(3))]
                ),
                indoc!(
                    "
                    {
                      x = if true {
                        #foo
                        none
                      } else {
                        none
                      }

                      y = none
                      none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_space_between_two_statement_comments() {
            assert_eq!(
                format_with_comments(
                    &Block::new(
                        vec![
                            Statement::new(
                                Some("x".into()),
                                Variable::new("none", Position::fake()),
                                line_position(3)
                            ),
                            Statement::new(
                                Some("y".into()),
                                Variable::new("none", Position::fake()),
                                line_position(6)
                            )
                        ],
                        Variable::new("none", line_position(7)),
                        Position::fake()
                    ),
                    &[
                        Comment::new("foo", line_position(2)),
                        Comment::new("bar", line_position(5))
                    ]
                ),
                indoc!(
                    "
                    {
                      #foo
                      x = none

                      #bar
                      y = none
                      none
                    }
                    "
                )
            );
        }
    }

    mod expression {
        use super::*;
        use pretty_assertions::assert_eq;

        fn format(expression: &Expression) -> String {
            mfmt::format(&compile_expression(&mut Context::new(&[]), expression))
        }

        fn format_with_comments(expression: &Expression, comments: &[Comment]) -> String {
            mfmt::format(&compile_expression(&mut Context::new(comments), expression))
        }

        #[test]
        fn format_broken_parent_expression() {
            assert_eq!(
                format(
                    &Call::new(
                        Variable::new("foo", Position::fake()),
                        vec![Call::new(
                            Variable::new("foo", Position::fake()),
                            vec![Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                line_position(2),
                            )
                            .into()],
                            line_position(1),
                        )
                        .into()],
                        Position::fake()
                    )
                    .into()
                ),
                indoc!(
                    "
                    foo(
                      foo(
                        1,
                      ),
                    )
                    "
                )
                .trim(),
            );
        }

        mod call {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn format_() {
                assert_eq!(
                    format(
                        &Call::new(
                            Variable::new("foo", Position::fake()),
                            vec![
                                Number::new(
                                    NumberRepresentation::FloatingPoint("1".into()),
                                    Position::fake()
                                )
                                .into(),
                                Number::new(
                                    NumberRepresentation::FloatingPoint("2".into()),
                                    Position::fake()
                                )
                                .into(),
                            ],
                            Position::fake()
                        )
                        .into()
                    ),
                    "foo(1, 2)"
                );
            }

            #[test]
            fn format_multi_line() {
                assert_eq!(
                    format(
                        &Call::new(
                            Variable::new("foo", line_position(1)),
                            vec![
                                Number::new(
                                    NumberRepresentation::FloatingPoint("1".into()),
                                    line_position(2)
                                )
                                .into(),
                                Number::new(
                                    NumberRepresentation::FloatingPoint("2".into()),
                                    Position::fake()
                                )
                                .into(),
                            ],
                            Position::fake()
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        foo(
                          1,
                          2,
                        )
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_block_comment() {
                assert_eq!(
                    format_with_comments(
                        &Call::new(
                            Variable::new("foo", line_position(1)),
                            vec![Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                line_position(3)
                            )
                            .into()],
                            Position::fake()
                        )
                        .into(),
                        &[Comment::new("foo", line_position(2))]
                    ),
                    indoc!(
                        "
                        foo(
                          #foo
                          1,
                        )
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_suffix_comment() {
                assert_eq!(
                    format_with_comments(
                        &Call::new(
                            Variable::new("foo", line_position(1)),
                            vec![Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                line_position(2)
                            )
                            .into()],
                            Position::fake()
                        )
                        .into(),
                        &[Comment::new("foo", line_position(2))]
                    ),
                    indoc!(
                        "
                        foo(
                          1, #foo
                        )
                        "
                    )
                    .trim()
                );
            }
        }

        mod if_ {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn format_single_line() {
                assert_eq!(
                    format(
                        &If::new(
                            vec![IfBranch::new(
                                Variable::new("true", Position::fake()),
                                Block::new(
                                    vec![],
                                    Variable::new("none", Position::fake()),
                                    Position::fake()
                                )
                            )],
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake()
                        )
                        .into()
                    ),
                    "if true { none } else { none }"
                );
            }

            #[test]
            fn format_multi_line_with_multi_line_input() {
                assert_eq!(
                    format(
                        &If::new(
                            vec![IfBranch::new(
                                Variable::new("true", Position::fake()),
                                Block::new(
                                    vec![],
                                    Variable::new("none", line_position(2)),
                                    Position::fake()
                                )
                            )],
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            line_position(1)
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        if true {
                          none
                        } else {
                          none
                        }
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_multi_line_with_multiple_branches() {
                assert_eq!(
                    format(
                        &If::new(
                            vec![
                                IfBranch::new(
                                    Variable::new("true", Position::fake()),
                                    Block::new(
                                        vec![],
                                        Variable::new("none", Position::fake()),
                                        Position::fake()
                                    )
                                ),
                                IfBranch::new(
                                    Variable::new("false", Position::fake()),
                                    Block::new(
                                        vec![],
                                        Variable::new("none", Position::fake()),
                                        Position::fake()
                                    )
                                )
                            ],
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake()
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        if true {
                          none
                        } else if false {
                          none
                        } else {
                          none
                        }
                        "
                    )
                    .trim()
                );
            }
        }

        #[test]
        fn format_if_list() {
            assert_eq!(
                format(
                    &IfList::new(
                        Variable::new("ys", Position::fake()),
                        "x",
                        "xs",
                        Block::new(
                            vec![],
                            Variable::new("x", Position::fake()),
                            Position::fake()
                        ),
                        Block::new(
                            vec![],
                            Variable::new("none", Position::fake()),
                            Position::fake()
                        ),
                        Position::fake()
                    )
                    .into()
                ),
                indoc!(
                    "
                    if [x, ...xs] = ys {
                      x
                    } else {
                      none
                    }
                    "
                )
                .trim()
            );
        }

        #[test]
        fn format_if_map() {
            assert_eq!(
                format(
                    &IfMap::new(
                        "x",
                        Variable::new("xs", Position::fake()),
                        Variable::new("k", Position::fake()),
                        Block::new(
                            vec![],
                            Variable::new("x", Position::fake()),
                            Position::fake()
                        ),
                        Block::new(
                            vec![],
                            Variable::new("none", Position::fake()),
                            Position::fake()
                        ),
                        Position::fake()
                    )
                    .into()
                ),
                indoc!(
                    "
                    if x = xs[k] {
                      x
                    } else {
                      none
                    }
                    "
                )
                .trim()
            );
        }

        mod if_type {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn format_single_line() {
                assert_eq!(
                    format(
                        &IfType::new(
                            "x",
                            Variable::new("y", Position::fake()),
                            vec![
                                IfTypeBranch::new(
                                    types::Reference::new("none", Position::fake()),
                                    Block::new(
                                        vec![],
                                        Variable::new("none", Position::fake()),
                                        Position::fake()
                                    )
                                ),
                                IfTypeBranch::new(
                                    types::Reference::new("number", Position::fake()),
                                    Block::new(
                                        vec![],
                                        Variable::new("none", Position::fake()),
                                        Position::fake()
                                    )
                                )
                            ],
                            None,
                            Position::fake(),
                        )
                        .into()
                    ),
                    "if x = y as none { none } else if number { none }"
                );
            }

            #[test]
            fn format_multi_line() {
                assert_eq!(
                    format(
                        &IfType::new(
                            "x",
                            Variable::new("y", Position::fake()),
                            vec![
                                IfTypeBranch::new(
                                    types::Reference::new("none", Position::fake()),
                                    Block::new(
                                        vec![],
                                        Variable::new("none", line_position(2)),
                                        Position::fake()
                                    )
                                ),
                                IfTypeBranch::new(
                                    types::Reference::new("number", Position::fake()),
                                    Block::new(
                                        vec![],
                                        Variable::new("none", Position::fake()),
                                        Position::fake()
                                    )
                                )
                            ],
                            None,
                            line_position(1),
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        if x = y as none {
                          none
                        } else if number {
                          none
                        }
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_with_else_block() {
                assert_eq!(
                    format(
                        &IfType::new(
                            "x",
                            Variable::new("y", Position::fake()),
                            vec![
                                IfTypeBranch::new(
                                    types::Reference::new("none", Position::fake()),
                                    Block::new(
                                        vec![],
                                        Variable::new("none", Position::fake()),
                                        Position::fake()
                                    )
                                ),
                                IfTypeBranch::new(
                                    types::Reference::new("number", Position::fake()),
                                    Block::new(
                                        vec![],
                                        Variable::new("none", Position::fake()),
                                        Position::fake()
                                    )
                                )
                            ],
                            Some(Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            )),
                            Position::fake(),
                        )
                        .into()
                    ),
                    indoc!(
                        "
                    if x = y as none {
                      none
                    } else if number {
                      none
                    } else {
                      none
                    }
                    "
                    )
                    .trim()
                );
            }
        }

        mod lambda {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn format_() {
                assert_eq!(
                    format(
                        &Lambda::new(
                            vec![],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake()
                        )
                        .into()
                    ),
                    "\\() none { none }"
                );
            }

            #[test]
            fn format_multi_line_body() {
                assert_eq!(
                    format(
                        &Lambda::new(
                            vec![],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![Statement::new(
                                    Some("x".into()),
                                    Variable::new("none", Position::fake()),
                                    Position::fake()
                                )],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake()
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        \\() none {
                          x = none
                          none
                        }
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_single_line_arguments_with_multi_line_body_of_expression() {
                assert_eq!(
                    format(
                        &Lambda::new(
                            vec![],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", line_position(2)),
                                Position::fake()
                            ),
                            line_position(1)
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        \\() none {
                          none
                        }
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_multi_line_argument() {
                assert_eq!(
                    format(
                        &Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Reference::new("none", Position::fake()),
                                line_position(2),
                            )],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            line_position(1)
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        \\(
                          x none,
                        ) none {
                          none
                        }
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_multi_line_arguments() {
                assert_eq!(
                    format(
                        &Lambda::new(
                            vec![
                                Argument::new(
                                    "x",
                                    types::Reference::new("none", Position::fake()),
                                    line_position(2)
                                ),
                                Argument::new(
                                    "y",
                                    types::Reference::new("none", Position::fake()),
                                    Position::fake()
                                )
                            ],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            line_position(1)
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        \\(
                          x none,
                          y none,
                        ) none {
                          none
                        }
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_suffix_comment_on_function_argument() {
                assert_eq!(
                    format_with_comments(
                        &Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Reference::new("none", Position::fake()),
                                line_position(2)
                            )],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake(),
                        )
                        .into(),
                        &[Comment::new("foo", line_position(2))]
                    ),
                    indoc!(
                        "
                        \\(
                          x none, #foo
                        ) none {
                          none
                        }
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_block_comment_on_function_argument() {
                assert_eq!(
                    format_with_comments(
                        &Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Reference::new("none", Position::fake()),
                                line_position(3),
                            )],
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake(),
                        )
                        .into(),
                        &[Comment::new("foo", line_position(2))]
                    ),
                    indoc!(
                        "
                        \\(
                          #foo
                          x none,
                        ) none {
                          none
                        }
                        "
                    )
                    .trim()
                );
            }
        }

        mod number {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn format_decimal_float() {
                assert_eq!(
                    format(
                        &Number::new(
                            NumberRepresentation::FloatingPoint("42".into()),
                            Position::fake()
                        )
                        .into()
                    ),
                    "42"
                );
            }

            #[test]
            fn format_binary() {
                assert_eq!(
                    format(
                        &Number::new(NumberRepresentation::Binary("01".into()), Position::fake())
                            .into()
                    ),
                    "0b01"
                );
            }

            #[test]
            fn format_hexadecimal() {
                assert_eq!(
                    format(
                        &Number::new(
                            NumberRepresentation::Hexadecimal("fa".into()),
                            Position::fake()
                        )
                        .into()
                    ),
                    "0xFA"
                );
            }
        }

        #[test]
        fn format_string() {
            assert_eq!(
                format(&ByteString::new("foo", Position::fake()).into()),
                "\"foo\""
            );
        }

        mod binary_operation {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn format_() {
                assert_eq!(
                    format(
                        &BinaryOperation::new(
                            BinaryOperator::Add,
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake()
                            ),
                            Number::new(
                                NumberRepresentation::FloatingPoint("2".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        )
                        .into()
                    ),
                    "1 + 2"
                );
            }

            #[test]
            fn format_multi_line() {
                assert_eq!(
                    format(
                        &BinaryOperation::new(
                            BinaryOperator::Add,
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                line_position(1)
                            ),
                            Number::new(
                                NumberRepresentation::FloatingPoint("2".into()),
                                line_position(2)
                            ),
                            Position::fake()
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        1
                          + 2
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_nested_operations() {
                assert_eq!(
                    format(
                        &BinaryOperation::new(
                            BinaryOperator::Add,
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake()
                            ),
                            BinaryOperation::new(
                                BinaryOperator::Multiply,
                                Number::new(
                                    NumberRepresentation::FloatingPoint("2".into()),
                                    Position::fake()
                                ),
                                Number::new(
                                    NumberRepresentation::FloatingPoint("3".into()),
                                    Position::fake()
                                ),
                                Position::fake()
                            ),
                            Position::fake()
                        )
                        .into()
                    ),
                    "1 + 2 * 3"
                );
            }

            #[test]
            fn format_nested_operations_with_priority() {
                assert_eq!(
                    format(
                        &BinaryOperation::new(
                            BinaryOperator::Multiply,
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake()
                            ),
                            BinaryOperation::new(
                                BinaryOperator::Add,
                                Number::new(
                                    NumberRepresentation::FloatingPoint("2".into()),
                                    Position::fake()
                                ),
                                Number::new(
                                    NumberRepresentation::FloatingPoint("3".into()),
                                    Position::fake()
                                ),
                                Position::fake()
                            ),
                            Position::fake()
                        )
                        .into()
                    ),
                    "1 * (2 + 3)"
                );
            }
        }

        mod unary_operation {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn format_not_operation() {
                assert_eq!(
                    format(
                        &UnaryOperation::new(
                            UnaryOperator::Not,
                            Variable::new("x", Position::fake()),
                            Position::fake()
                        )
                        .into()
                    ),
                    "!x"
                );
            }

            #[test]
            fn format_try_operation() {
                assert_eq!(
                    format(
                        &UnaryOperation::new(
                            UnaryOperator::Try,
                            Variable::new("x", Position::fake()),
                            Position::fake()
                        )
                        .into()
                    ),
                    "x?"
                );
            }

            #[test]
            fn format_with_binary_operation() {
                assert_eq!(
                    format(
                        &UnaryOperation::new(
                            UnaryOperator::Not,
                            BinaryOperation::new(
                                BinaryOperator::And,
                                Variable::new("true", Position::fake()),
                                Variable::new("false", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake()
                        )
                        .into(),
                    ),
                    "!(true & false)"
                );
            }
        }

        #[test]
        fn format_record_deconstruction() {
            assert_eq!(
                format(
                    &RecordDeconstruction::new(
                        Variable::new("x", Position::fake()),
                        "y",
                        Position::fake()
                    )
                    .into()
                ),
                "x.y"
            );
        }

        mod list {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn format_empty() {
                assert_eq!(
                    format(
                        &List::new(
                            types::Reference::new("none", Position::fake()),
                            vec![],
                            Position::fake()
                        )
                        .into()
                    ),
                    "[none]"
                );
            }

            #[test]
            fn format_element() {
                assert_eq!(
                    format(
                        &List::new(
                            types::Reference::new("none", Position::fake()),
                            vec![ListElement::Single(
                                Variable::new("none", Position::fake()).into()
                            )],
                            Position::fake()
                        )
                        .into()
                    ),
                    "[none none]"
                );
            }

            #[test]
            fn format_two_elements() {
                assert_eq!(
                    format(
                        &List::new(
                            types::Reference::new("none", Position::fake()),
                            vec![
                                ListElement::Single(Variable::new("none", Position::fake()).into()),
                                ListElement::Single(Variable::new("none", Position::fake()).into())
                            ],
                            Position::fake()
                        )
                        .into()
                    ),
                    "[none none, none]"
                );
            }

            #[test]
            fn format_multi_line() {
                assert_eq!(
                    format(
                        &List::new(
                            types::Reference::new("none", Position::fake()),
                            vec![ListElement::Single(
                                Variable::new("none", line_position(2)).into()
                            )],
                            line_position(1)
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        [none
                          none,
                        ]
                        "
                    )
                    .trim()
                );
            }

            #[test]
            fn format_multi_line_with_two_elements() {
                assert_eq!(
                    format(
                        &List::new(
                            types::Reference::new("number", Position::fake()),
                            vec![
                                ListElement::Single(
                                    Number::new(
                                        NumberRepresentation::FloatingPoint("1".into()),
                                        line_position(2)
                                    )
                                    .into()
                                ),
                                ListElement::Single(
                                    Number::new(
                                        NumberRepresentation::FloatingPoint("2".into()),
                                        Position::fake()
                                    )
                                    .into()
                                )
                            ],
                            line_position(1)
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        [number
                          1,
                          2,
                        ]
                        "
                    )
                    .trim()
                );
            }

            mod list_comprehension {
                use super::*;
                use pretty_assertions::assert_eq;

                #[test]
                fn format_comprehension() {
                    assert_eq!(
                        format(
                            &ListComprehension::new(
                                types::Reference::new("none", Position::fake()),
                                Variable::new("none", Position::fake()),
                                vec![ListComprehensionBranch::new(
                                    vec!["x".into()],
                                    vec![Variable::new("xs", Position::fake()).into()],
                                    None,
                                    Position::fake(),
                                )],
                                Position::fake(),
                            )
                            .into()
                        ),
                        "[none none for x in xs]"
                    );
                }

                #[test]
                fn format_multi_line_comprehension() {
                    assert_eq!(
                        format(
                            &ListComprehension::new(
                                types::Reference::new("none", Position::fake()),
                                Variable::new("none", line_position(2)),
                                vec![ListComprehensionBranch::new(
                                    vec!["x".into()],
                                    vec![Variable::new("xs", Position::fake()).into()],
                                    None,
                                    line_position(2),
                                )],
                                line_position(1),
                            )
                            .into()
                        ),
                        indoc!(
                            "
                            [none
                              none
                              for x in xs
                            ]
                            "
                        )
                        .trim()
                    );
                }

                #[test]
                fn format_comprehension_with_condition() {
                    assert_eq!(
                        format(
                            &ListComprehension::new(
                                types::Reference::new("none", Position::fake()),
                                Variable::new("none", line_position(2)),
                                vec![ListComprehensionBranch::new(
                                    vec!["x".into()],
                                    vec![Variable::new("xs", Position::fake()).into()],
                                    Some(Variable::new("true", Position::fake()).into()),
                                    line_position(2)
                                )],
                                line_position(1)
                            )
                            .into()
                        ),
                        indoc!(
                            "
                            [none
                              none
                              for x in xs if true
                            ]
                            "
                        )
                        .trim()
                    );
                }

                #[test]
                fn format_parallel() {
                    assert_eq!(
                        format(
                            &ListComprehension::new(
                                types::Reference::new("none", Position::fake()),
                                Variable::new("none", line_position(2)),
                                vec![ListComprehensionBranch::new(
                                    vec!["x".into(), "y".into()],
                                    vec![
                                        Variable::new("xs", Position::fake()).into(),
                                        Variable::new("ys", Position::fake()).into()
                                    ],
                                    None,
                                    line_position(2)
                                )],
                                line_position(1)
                            )
                            .into()
                        ),
                        indoc!(
                            "
                            [none
                              none
                              for x, y in xs, ys
                            ]
                            "
                        )
                        .trim()
                    );
                }
            }
        }

        mod map {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn format_empty() {
                assert_eq!(
                    format(
                        &Map::new(
                            types::Reference::new("string", Position::fake()),
                            types::Reference::new("number", Position::fake()),
                            vec![],
                            Position::fake()
                        )
                        .into()
                    ),
                    "{string: number}"
                );
            }

            #[test]
            fn format_entry() {
                assert_eq!(
                    format(
                        &Map::new(
                            types::Reference::new("string", Position::fake()),
                            types::Reference::new("number", Position::fake()),
                            vec![MapEntry::new(
                                ByteString::new("foo", Position::fake()),
                                Number::new(
                                    NumberRepresentation::FloatingPoint("42".into()),
                                    Position::fake()
                                ),
                                Position::fake()
                            )
                            .into()],
                            Position::fake()
                        )
                        .into()
                    ),
                    "{string: number \"foo\": 42}"
                );
            }

            #[test]
            fn format_two_entries() {
                assert_eq!(
                    format(
                        &Map::new(
                            types::Reference::new("string", Position::fake()),
                            types::Reference::new("number", Position::fake()),
                            vec![
                                MapEntry::new(
                                    ByteString::new("foo", Position::fake()),
                                    Number::new(
                                        NumberRepresentation::FloatingPoint("1".into()),
                                        Position::fake()
                                    ),
                                    Position::fake()
                                )
                                .into(),
                                MapEntry::new(
                                    ByteString::new("bar", Position::fake()),
                                    Number::new(
                                        NumberRepresentation::FloatingPoint("2".into()),
                                        Position::fake()
                                    ),
                                    Position::fake()
                                )
                                .into()
                            ],
                            Position::fake()
                        )
                        .into()
                    ),
                    "{string: number \"foo\": 1, \"bar\": 2}"
                );
            }

            #[test]
            fn format_map() {
                assert_eq!(
                    format(
                        &Map::new(
                            types::Reference::new("string", Position::fake()),
                            types::Reference::new("number", Position::fake()),
                            vec![MapElement::Multiple(
                                Variable::new("xs", Position::fake()).into()
                            )],
                            Position::fake()
                        )
                        .into()
                    ),
                    "{string: number ...xs}"
                );
            }

            #[test]
            fn format_multi_line() {
                assert_eq!(
                    format(
                        &Map::new(
                            types::Reference::new("string", Position::fake()),
                            types::Reference::new("number", Position::fake()),
                            vec![MapEntry::new(
                                ByteString::new("foo", Position::fake()),
                                Number::new(
                                    NumberRepresentation::FloatingPoint("1".into()),
                                    Position::fake()
                                ),
                                line_position(2)
                            )
                            .into()],
                            line_position(1)
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        {string: number
                          \"foo\": 1,
                        }
                        "
                    )
                    .trim(),
                );
            }

            #[test]
            fn format_multi_line_with_two_entries() {
                assert_eq!(
                    format(
                        &Map::new(
                            types::Reference::new("string", Position::fake()),
                            types::Reference::new("number", Position::fake()),
                            vec![
                                MapEntry::new(
                                    ByteString::new("foo", Position::fake()),
                                    Number::new(
                                        NumberRepresentation::FloatingPoint("1".into()),
                                        Position::fake()
                                    ),
                                    line_position(2)
                                )
                                .into(),
                                MapEntry::new(
                                    ByteString::new("bar", Position::fake()),
                                    Number::new(
                                        NumberRepresentation::FloatingPoint("2".into()),
                                        Position::fake()
                                    ),
                                    Position::fake()
                                )
                                .into()
                            ],
                            line_position(1)
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        {string: number
                          \"foo\": 1,
                          \"bar\": 2,
                        }
                        "
                    )
                    .trim(),
                );
            }
        }

        mod record {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn format_empty() {
                assert_eq!(
                    format(&Record::new("foo", None, vec![], Position::fake()).into()),
                    "foo{}"
                );
            }

            #[test]
            fn format_field() {
                assert_eq!(
                    format(
                        &Record::new(
                            "foo",
                            None,
                            vec![RecordField::new(
                                "x",
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            )],
                            Position::fake()
                        )
                        .into()
                    ),
                    "foo{x: none}"
                );
            }

            #[test]
            fn format_two_fields() {
                assert_eq!(
                    format(
                        &Record::new(
                            "foo",
                            None,
                            vec![
                                RecordField::new(
                                    "x",
                                    Number::new(
                                        NumberRepresentation::FloatingPoint("1".into()),
                                        Position::fake()
                                    ),
                                    Position::fake()
                                ),
                                RecordField::new(
                                    "y",
                                    Number::new(
                                        NumberRepresentation::FloatingPoint("2".into()),
                                        Position::fake()
                                    ),
                                    Position::fake()
                                )
                            ],
                            Position::fake()
                        )
                        .into()
                    ),
                    "foo{x: 1, y: 2}"
                );
            }

            #[test]
            fn format_update() {
                assert_eq!(
                    format(
                        &Record::new(
                            "foo",
                            Some(Variable::new("r", Position::fake()).into()),
                            vec![RecordField::new(
                                "x",
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            )],
                            Position::fake()
                        )
                        .into()
                    ),
                    "foo{...r, x: none}"
                );
            }

            #[test]
            fn format_multi_line_update_with_entry_on_next_line() {
                assert_eq!(
                    format(
                        &Record::new(
                            "foo",
                            Some(Variable::new("r", Position::fake()).into()),
                            vec![RecordField::new(
                                "x",
                                Variable::new("none", Position::fake()),
                                line_position(2)
                            )],
                            line_position(1)
                        )
                        .into()
                    ),
                    "foo{...r, x: none}"
                );
            }

            #[test]
            fn format_multi_line() {
                assert_eq!(
                    format(
                        &Record::new(
                            "foo",
                            None,
                            vec![RecordField::new(
                                "x",
                                Variable::new("none", Position::fake()),
                                line_position(2)
                            )],
                            line_position(1)
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        foo{
                          x: none,
                        }
                        "
                    )
                    .trim(),
                );
            }

            #[test]
            fn format_multi_line_with_two_fields() {
                assert_eq!(
                    format(
                        &Record::new(
                            "foo",
                            None,
                            vec![
                                RecordField::new(
                                    "x",
                                    Variable::new("none", Position::fake()),
                                    line_position(2)
                                ),
                                RecordField::new(
                                    "y",
                                    Variable::new("none", Position::fake()),
                                    line_position(2)
                                )
                            ],
                            line_position(1)
                        )
                        .into()
                    ),
                    indoc!(
                        "
                        foo{
                          x: none,
                          y: none,
                        }
                        "
                    )
                    .trim(),
                );
            }

            #[test]
            fn format_entry_with_block_comment() {
                assert_eq!(
                    format_with_comments(
                        &Record::new(
                            "foo",
                            None,
                            vec![RecordField::new(
                                "x",
                                Variable::new("none", Position::fake()),
                                line_position(3)
                            )],
                            line_position(1)
                        )
                        .into(),
                        &[Comment::new("foo", line_position(2))]
                    ),
                    indoc!(
                        "
                        foo{
                          #foo
                          x: none,
                        }
                        "
                    )
                    .trim(),
                );
            }

            #[test]
            fn format_entry_with_suffix_comment() {
                assert_eq!(
                    format_with_comments(
                        &Record::new(
                            "foo",
                            None,
                            vec![RecordField::new(
                                "x",
                                Variable::new("none", Position::fake()),
                                line_position(2)
                            )],
                            line_position(1)
                        )
                        .into(),
                        &[Comment::new("foo", line_position(2))]
                    ),
                    indoc!(
                        "
                        foo{
                          x: none, #foo
                        }
                        "
                    )
                    .trim(),
                );
            }

            #[test]
            fn format_update_with_block_comment() {
                assert_eq!(
                    format_with_comments(
                        &Record::new(
                            "foo",
                            Some(Variable::new("x", line_position(3)).into()),
                            vec![],
                            line_position(1)
                        )
                        .into(),
                        &[Comment::new("foo", line_position(2))]
                    ),
                    indoc!(
                        "
                        foo{
                          #foo
                          ...x,
                        }
                        "
                    )
                    .trim(),
                );
            }

            #[test]
            fn format_update_with_suffix_comment() {
                assert_eq!(
                    format_with_comments(
                        &Record::new(
                            "foo",
                            Some(Variable::new("x", line_position(2)).into()),
                            vec![],
                            line_position(1)
                        )
                        .into(),
                        &[Comment::new("foo", line_position(2))]
                    ),
                    indoc!(
                        "
                        foo{
                          ...x, #foo
                        }
                        "
                    )
                    .trim(),
                );
            }
        }
    }

    mod comment {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn format_comment() {
            assert_eq!(
                format(
                    &Module::new(vec![], vec![], vec![], vec![], Position::fake()),
                    &[Comment::new("foo", Position::fake())]
                ),
                "#foo\n"
            );
        }

        #[test]
        fn keep_spaces_between_comments() {
            assert_eq!(
                format(
                    &Module::new(vec![], vec![], vec![], vec![], Position::fake()),
                    &[
                        Comment::new("foo", line_position(1)),
                        Comment::new("bar", line_position(3)),
                    ]
                ),
                indoc!(
                    "
                    #foo

                    #bar
                    ",
                ),
            );
        }

        #[test]
        fn format_comment_after_last_section() {
            assert_eq!(
                format(
                    &Module::new(
                        vec![Import::new(
                            InternalModulePath::new(vec!["Foo".into()]),
                            None,
                            vec![],
                            line_position(1),
                        )],
                        vec![],
                        vec![],
                        vec![],
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(2))]
                ),
                indoc!(
                    "
                    import 'Foo

                    #foo
                    ",
                ),
            );
        }

        #[test]
        fn keep_spaces_between_comment_and_next_line() {
            assert_eq!(
                format(
                    &Module::new(
                        vec![Import::new(
                            InternalModulePath::new(vec!["Foo".into()]),
                            None,
                            vec![],
                            line_position(3),
                        )],
                        vec![],
                        vec![],
                        vec![],
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    #foo

                    import 'Foo
                    ",
                ),
            );
        }

        #[test]
        fn format_import() {
            assert_eq!(
                format(
                    &Module::new(
                        vec![Import::new(
                            InternalModulePath::new(vec!["Foo".into()]),
                            None,
                            vec![],
                            line_position(2),
                        )],
                        vec![],
                        vec![],
                        vec![],
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    #foo
                    import 'Foo
                    ",
                ),
            );
        }

        #[test]
        fn format_foreign_import() {
            assert_eq!(
                format(
                    &Module::new(
                        vec![],
                        vec![ForeignImport::new(
                            "foo",
                            CallingConvention::Native,
                            types::Function::new(
                                vec![],
                                types::Reference::new("none", Position::fake()),
                                Position::fake()
                            ),
                            line_position(2),
                        )],
                        vec![],
                        vec![],
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    #foo
                    import foreign foo \\() none
                    ",
                ),
            );
        }

        #[test]
        fn format_record_definition() {
            assert_eq!(
                format(
                    &Module::new(
                        vec![],
                        vec![],
                        vec![RecordDefinition::new("foo", vec![], line_position(2)).into()],
                        vec![],
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    #foo
                    type foo {}
                    "
                )
            );
        }

        #[test]
        fn format_suffix_comment_on_record_field() {
            assert_eq!(
                format(
                    &Module::new(
                        vec![],
                        vec![],
                        vec![RecordDefinition::new(
                            "foo",
                            vec![types::RecordField::new(
                                "bar",
                                types::Reference::new("none", Position::fake()),
                                line_position(2),
                            )],
                            line_position(1)
                        )
                        .into()],
                        vec![],
                        Position::fake()
                    ),
                    &[Comment::new("comment", line_position(2))]
                ),
                indoc!(
                    "
                    type foo {
                      bar none #comment
                    }
                    "
                )
            );
        }

        #[test]
        fn format_block_comment_on_record_field() {
            assert_eq!(
                format(
                    &Module::new(
                        vec![],
                        vec![],
                        vec![RecordDefinition::new(
                            "foo",
                            vec![types::RecordField::new(
                                "bar",
                                types::Reference::new("none", Position::fake()),
                                line_position(3),
                            )],
                            line_position(1)
                        )
                        .into()],
                        vec![],
                        Position::fake()
                    ),
                    &[Comment::new("comment", line_position(2))]
                ),
                indoc!(
                    "
                    type foo {
                      #comment
                      bar none
                    }
                    "
                )
            );
        }

        #[test]
        fn format_type_alias() {
            assert_eq!(
                format(
                    &Module::new(
                        vec![],
                        vec![],
                        vec![TypeAlias::new(
                            "foo",
                            types::Reference::new("none", Position::fake()),
                            line_position(2)
                        )
                        .into()],
                        vec![],
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    #foo
                    type foo = none
                    "
                ),
            );
        }

        #[test]
        fn format_function_definition() {
            assert_eq!(
                format(
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
                                Position::fake(),
                            ),
                            None,
                            line_position(2)
                        )],
                        Position::fake()
                    ),
                    &[Comment::new("foo", line_position(1))]
                ),
                indoc!(
                    "
                    #foo
                    foo = \\() none { none }
                    "
                ),
            );
        }
    }
}
