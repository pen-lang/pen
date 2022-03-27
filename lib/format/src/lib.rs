use ast::{types::Type, *};
use std::str;

const INDENT_DEPTH: usize = 2;

// TODO Consider introducing a minimum editor width to enforce single-line
// formats in some occasions.

pub fn format(module: &Module, _comments: &[Comment]) -> String {
    let (external_imports, internal_imports) = module
        .imports()
        .iter()
        .partition::<Vec<_>, _>(|import| matches!(import.module_path(), ModulePath::External(_)));

    format_spaces(
        [
            format_imports(&external_imports),
            format_imports(&internal_imports),
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
            + "\n",
    )
}

fn format_spaces(string: impl AsRef<str>) -> String {
    regex::Regex::new(r"[ \t]*\n")
        .unwrap()
        .replace_all(string.as_ref(), "\n")
        .into()
}

fn format_imports(imports: &[&Import]) -> String {
    let mut imports = imports
        .iter()
        .map(|import| format_import(import))
        .collect::<Vec<_>>();

    imports.sort();

    imports.join("\n")
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
                .map(|field| indent(field.name().to_owned() + " " + &format_type(field.type_())))
                .collect::<Vec<_>>()
                .join("\n"),
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
            ["foreign"]
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
    let arguments = lambda
        .arguments()
        .iter()
        .map(|argument| format!("{} {}", argument.name(), format_type(argument.type_())))
        .collect::<Vec<_>>();
    let single_line_arguments = arguments.is_empty()
        || Some(lambda.position().line_number())
            == lambda
                .arguments()
                .get(0)
                .map(|argument| argument.type_().position().line_number())
            && arguments.iter().all(is_single_line);

    format!(
        "\\{} {} {}",
        if single_line_arguments {
            "(".to_owned() + &arguments.join(", ") + ")"
        } else {
            ["(".into()]
                .into_iter()
                .chain(arguments.into_iter().map(|argument| indent(argument) + ","))
                .chain([")".into()])
                .collect::<Vec<_>>()
                .join("\n")
        },
        format_type(lambda.result_type()),
        if single_line_arguments
            && lambda.position().line_number()
                == lambda.body().expression().position().line_number()
        {
            format_block(lambda.body())
        } else {
            format_multi_line_block(lambda.body())
        }
    )
}

fn format_block(block: &Block) -> String {
    let expression = format_expression(block.expression());

    if block.statements().is_empty() && is_single_line(&expression) {
        ["{", &expression, "}"].join(" ")
    } else {
        format_multi_line_block(block)
    }
}

fn format_multi_line_block(block: &Block) -> String {
    ["{".into()]
        .into_iter()
        .chain(
            block
                .statements()
                .iter()
                .zip(
                    block
                        .statements()
                        .iter()
                        .skip(1)
                        .map(|statement| statement.position())
                        .chain([block.expression().position()])
                        .map(|position| position.line_number()),
                )
                .map(|(current, next_line_number)| {
                    // TODO Use end positions of spans when they are available.
                    let line_count = next_line_number - current.position().line_number();
                    let current = format_statement(current);

                    if count_lines(&current) >= line_count {
                        current
                    } else {
                        current + "\n"
                    }
                })
                .map(indent),
        )
        .chain([indent(format_expression(block.expression()))])
        .chain(["}".into()])
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_statement(statement: &Statement) -> String {
    statement
        .name()
        .map(|name| format!("{} =", name))
        .into_iter()
        .chain([format_expression(statement.expression())])
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_expression(expression: &Expression) -> String {
    match expression {
        Expression::BinaryOperation(operation) => format_binary_operation(operation),
        Expression::Call(call) => {
            let head = format!("{}(", format_expression(call.function()));
            let arguments = call
                .arguments()
                .iter()
                .map(format_expression)
                .collect::<Vec<_>>();

            if call.arguments().is_empty()
                || Some(call.function().position().line_number())
                    == call
                        .arguments()
                        .get(0)
                        .map(|expression| expression.position().line_number())
                    && arguments.iter().all(is_single_line)
            {
                [head]
                    .into_iter()
                    .chain(if call.arguments().is_empty() {
                        None
                    } else {
                        Some(arguments.join(", "))
                    })
                    .chain([")".into()])
                    .collect::<Vec<_>>()
                    .concat()
            } else {
                [head]
                    .into_iter()
                    .chain(
                        call.arguments()
                            .iter()
                            .map(|expression| indent(format_expression(expression)) + ","),
                    )
                    .chain([")".into()])
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        Expression::Boolean(boolean) => if boolean.value() { "true" } else { "false" }.into(),
        Expression::If(if_) => format_if(if_),
        Expression::IfList(if_) => [
            "if".into(),
            format!("[{}, ...{}]", if_.first_name(), if_.rest_name()),
            "=".into(),
            format_expression(if_.list()),
            format_multi_line_block(if_.then()),
            "else".into(),
            format_multi_line_block(if_.else_()),
        ]
        .join(" "),
        Expression::IfMap(if_) => [
            "if".into(),
            if_.name().into(),
            "=".into(),
            format!(
                "{}[{}]",
                format_expression(if_.map()),
                format_expression(if_.key())
            ),
            format_multi_line_block(if_.then()),
            "else".into(),
            format_multi_line_block(if_.else_()),
        ]
        .join(" "),
        Expression::IfType(if_) => format_if_type(if_),
        Expression::Lambda(lambda) => format_lambda(lambda),
        Expression::List(list) => {
            let type_ = format_type(list.type_());
            let elements = list
                .elements()
                .iter()
                .map(format_list_element)
                .collect::<Vec<_>>();

            if elements.is_empty()
                || Some(list.position().line_number())
                    == list
                        .elements()
                        .get(0)
                        .map(|element| element.position().line_number())
                    && elements.iter().all(is_single_line)
            {
                ["[".into()]
                    .into_iter()
                    .chain([[type_]
                        .into_iter()
                        .chain(if elements.is_empty() {
                            None
                        } else {
                            Some(elements.join(", "))
                        })
                        .collect::<Vec<_>>()
                        .join(" ")])
                    .chain(["]".into()])
                    .collect::<Vec<_>>()
                    .concat()
            } else {
                [format!("[{}", type_)]
                    .into_iter()
                    .chain(elements.into_iter().map(|element| indent(element) + ","))
                    .chain(["]".into()])
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        Expression::ListComprehension(comprehension) => {
            let type_ = format_type(comprehension.type_());
            let element = format_expression(comprehension.element());
            let list = format_expression(comprehension.list());

            if comprehension.position().line_number()
                == comprehension.element().position().line_number()
                && is_single_line(&element)
                && is_single_line(&list)
            {
                ["[".into()]
                    .into_iter()
                    .chain([[
                        type_,
                        element,
                        "for".into(),
                        comprehension.element_name().into(),
                        "in".into(),
                        list,
                    ]
                    .join(" ")])
                    .chain(["]".into()])
                    .collect::<Vec<_>>()
                    .concat()
            } else {
                [format!("[{}", type_)]
                    .into_iter()
                    .chain(
                        [
                            element,
                            format!("for {} in {}", comprehension.element_name(), list),
                        ]
                        .iter()
                        .map(indent),
                    )
                    .chain(["]".into()])
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        Expression::Map(map) => {
            let type_ = format_type(map.key_type()) + ": " + &format_type(map.value_type());
            let elements = map
                .elements()
                .iter()
                .map(|element| match element {
                    MapElement::Map(expression) => {
                        format!("...{}", format_expression(expression))
                    }
                    MapElement::Insertion(entry) => {
                        format!(
                            "{}: {}",
                            format_expression(entry.key()),
                            format_expression(entry.value())
                        )
                    }
                    MapElement::Removal(expression) => format_expression(expression),
                })
                .collect::<Vec<_>>();

            if elements.is_empty()
                || Some(map.position().line_number())
                    == map
                        .elements()
                        .get(0)
                        .map(|element| element.position().line_number())
                    && elements.iter().all(is_single_line)
            {
                ["{".into()]
                    .into_iter()
                    .chain([[type_]
                        .into_iter()
                        .chain(if map.elements().is_empty() {
                            None
                        } else {
                            Some(elements.join(", "))
                        })
                        .collect::<Vec<_>>()
                        .join(" ")])
                    .chain(["}".into()])
                    .collect::<Vec<_>>()
                    .concat()
            } else {
                [format!("{{{}", type_)]
                    .into_iter()
                    .chain(elements.into_iter().map(|element| indent(element) + ","))
                    .chain(["}".into()])
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        Expression::None(_) => "none".into(),
        Expression::Number(number) => match number.value() {
            NumberRepresentation::Binary(string) => "0b".to_owned() + string,
            NumberRepresentation::Hexadecimal(string) => "0x".to_owned() + &string.to_uppercase(),
            NumberRepresentation::FloatingPoint(string) => string.clone(),
        },
        Expression::Record(record) => {
            let elements = record
                .record()
                .map(|expression| format!("...{}", format_expression(expression)))
                .into_iter()
                .chain(record.fields().iter().map(|field| {
                    format!(
                        "{}: {}",
                        field.name(),
                        format_expression(field.expression())
                    )
                }))
                .collect::<Vec<_>>();

            if record.fields().is_empty()
                || Some(record.position().line_number())
                    == record
                        .fields()
                        .get(0)
                        .map(|field| field.position().line_number())
                    && elements.iter().all(is_single_line)
            {
                [record.type_name().into(), "{".into()]
                    .into_iter()
                    .chain(if record.record().is_none() && record.fields().is_empty() {
                        None
                    } else {
                        Some(elements.join(", "))
                    })
                    .chain(["}".into()])
                    .collect::<Vec<_>>()
                    .concat()
            } else {
                [record.type_name().to_owned() + "{"]
                    .into_iter()
                    .chain(elements.into_iter().map(|line| indent(line) + ","))
                    .chain(["}".into()])
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        Expression::RecordDeconstruction(deconstruction) => format!(
            "{}.{}",
            format_expression(deconstruction.expression()),
            deconstruction.name()
        ),
        Expression::SpawnOperation(operation) => {
            format!("go {}", format_lambda(operation.function()))
        }
        Expression::String(string) => {
            format!("\"{}\"", string.value())
        }
        Expression::UnaryOperation(operation) => {
            let operand = format_expression(operation.expression());
            let operand = if matches!(operation.expression(), Expression::BinaryOperation(_)) {
                "(".to_owned() + &operand + ")"
            } else {
                operand
            };

            match operation.operator() {
                UnaryOperator::Not => "!".to_owned() + &operand,
                UnaryOperator::Try => operand + "?",
            }
        }
        Expression::Variable(variable) => variable.name().into(),
    }
}

fn format_if(if_: &If) -> String {
    let branches = if_
        .branches()
        .iter()
        .map(|branch| format_expression(branch.condition()) + " " + &format_block(branch.block()))
        .collect::<Vec<_>>();
    let else_ = format_block(if_.else_());

    if branches.len() == 1
        && branches.iter().chain([&else_]).all(is_single_line)
        && Some(if_.position().line_number())
            == if_
                .branches()
                .get(0)
                .map(|branch| branch.block().expression().position().line_number())
    {
        branches
            .into_iter()
            .flat_map(|branch| ["if".into(), branch, "else".into()])
            .chain([else_])
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        if_.branches()
            .iter()
            .flat_map(|branch| {
                [
                    "if".into(),
                    format_expression(branch.condition()),
                    format_multi_line_block(branch.block()),
                    "else".into(),
                ]
            })
            .chain([format_multi_line_block(if_.else_())])
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn format_if_type(if_: &IfType) -> String {
    let argument = format_expression(if_.argument());
    let branches = if_
        .branches()
        .iter()
        .map(|branch| format_type(branch.type_()) + " " + &format_block(branch.block()))
        .collect::<Vec<_>>();
    let else_ = if_.else_().map(format_block);

    let head = [
        "if".into(),
        if_.name().into(),
        "=".into(),
        argument.clone(),
        "as".into(),
    ];

    if branches.iter().chain(&else_).count() <= 2
        && [&argument]
            .into_iter()
            .chain(&branches)
            .chain(else_.as_ref())
            .all(is_single_line)
        && Some(if_.position().line_number())
            == if_
                .branches()
                .get(0)
                .map(|branch| branch.block().expression().position().line_number())
    {
        head.into_iter()
            .chain([branches.join(" else if ")])
            .chain(else_.into_iter().flat_map(|block| ["else".into(), block]))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        head.into_iter()
            .chain([if_
                .branches()
                .iter()
                .map(|branch| {
                    format_type(branch.type_()) + " " + &format_multi_line_block(branch.block())
                })
                .collect::<Vec<_>>()
                .join(" else if ")])
            .chain(
                if_.else_()
                    .iter()
                    .flat_map(|block| ["else".into(), format_multi_line_block(block)]),
            )
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn format_list_element(element: &ListElement) -> String {
    match element {
        ListElement::Multiple(expression) => {
            format!("...{}", format_expression(expression))
        }
        ListElement::Single(expression) => format_expression(expression),
    }
}

fn format_binary_operation(operation: &BinaryOperation) -> String {
    let single_line =
        operation.lhs().position().line_number() == operation.rhs().position().line_number();
    let operator = format_binary_operator(operation.operator()).into();

    [
        format_operand(operation.lhs(), operation.operator()),
        [
            if single_line {
                operator
            } else {
                indent(operator)
            },
            format_operand(operation.rhs(), operation.operator()),
        ]
        .join(" "),
    ]
    .join(if single_line { " " } else { "\n" })
}

fn format_operand(operand: &Expression, parent_operator: BinaryOperator) -> String {
    let string = format_expression(operand);

    if match operand {
        Expression::BinaryOperation(operation) => Some(operation),
        _ => None,
    }
    .map(|operand| operator_priority(operand.operator()) < operator_priority(parent_operator))
    .unwrap_or_default()
    {
        format!("({})", string)
    } else {
        string
    }
}

fn format_binary_operator(operator: BinaryOperator) -> &'static str {
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
}

fn operator_priority(operator: BinaryOperator) -> usize {
    match operator {
        BinaryOperator::Or => 1,
        BinaryOperator::And => 2,
        BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::LessThan
        | BinaryOperator::LessThanOrEqual
        | BinaryOperator::GreaterThan
        | BinaryOperator::GreaterThanOrEqual => 3,
        BinaryOperator::Add | BinaryOperator::Subtract => 4,
        BinaryOperator::Multiply | BinaryOperator::Divide => 5,
    }
}

fn indent(string: impl AsRef<str>) -> String {
    regex::Regex::new("^|\n")
        .unwrap()
        .replace_all(
            string.as_ref(),
            "${0}".to_owned() + &" ".repeat(INDENT_DEPTH),
        )
        .into()
}

fn count_lines(string: &str) -> usize {
    string.trim().matches('\n').count() + 1
}

fn is_single_line(string: impl AsRef<str>) -> bool {
    !string.as_ref().contains('\n')
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use position::{test::PositionFake, Position};

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
            "\n"
        );
    }

    mod import {
        use super::*;

        #[test]
        fn format_internal_module_import() {
            assert_eq!(
                format_module(&Module::new(
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
                format_module(&Module::new(
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
                format_module(&Module::new(
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
                format_module(&Module::new(
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
        fn sort_module_imports_with_external_paths() {
            assert_eq!(
                format_module(&Module::new(
                    vec![
                        Import::new(
                            ExternalModulePath::new("Foo", vec!["Foo".into()]),
                            None,
                            vec![],
                        ),
                        Import::new(
                            ExternalModulePath::new("Bar", vec!["Bar".into()]),
                            None,
                            vec![]
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
                        Import::new(InternalModulePath::new(vec!["Foo".into()]), None, vec![],),
                        Import::new(InternalModulePath::new(vec!["Bar".into()]), None, vec![])
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
                        ),
                        Import::new(
                            ExternalModulePath::new("Package", vec!["Foo".into(), "Bar".into()]),
                            None,
                            vec![]
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
            format_module(&Module::new(
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
                        types::None::new(Position::fake())
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
                        types::RecordField::new("foo", types::None::new(Position::fake())),
                        types::RecordField::new("bar", types::None::new(Position::fake()))
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

    #[test]
    fn format_type_alias() {
        assert_eq!(
            format_module(&Module::new(
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

    mod definition {
        use super::*;

        #[test]
        fn format_with_no_argument_and_no_statement() {
            assert_eq!(
                format_module(&Module::new(
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
        fn format_with_argument() {
            assert_eq!(
                format_module(&Module::new(
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
                    vec![Definition::new(
                        "foo",
                        Lambda::new(
                            vec![],
                            types::Function::new(
                                vec![],
                                types::None::new(Position::fake()),
                                Position::fake()
                            ),
                            Block::new(
                                vec![],
                                Lambda::new(
                                    vec![],
                                    types::None::new(Position::fake()),
                                    Block::new(
                                        vec![],
                                        None::new(Position::fake()),
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
                    vec![Definition::new(
                        "foo",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
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
                    vec![Definition::new(
                        "foo",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
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

        #[test]
        fn format_() {
            assert_eq!(
                format_block(&Block::new(
                    vec![],
                    None::new(Position::fake()),
                    Position::fake()
                )),
                "{ none }"
            );
        }

        #[test]
        fn format_statement() {
            assert_eq!(
                format_block(&Block::new(
                    vec![Statement::new(
                        None,
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    None::new(Position::fake()),
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
                .trim()
            );
        }

        #[test]
        fn format_statement_with_name() {
            assert_eq!(
                format_block(&Block::new(
                    vec![Statement::new(
                        Some("x".into()),
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    None::new(Position::fake()),
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
                .trim()
            );
        }

        #[test]
        fn format_statement_with_no_blank_line() {
            assert_eq!(
                format_block(&Block::new(
                    vec![Statement::new(
                        None,
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        line_position(1)
                    )],
                    None::new(line_position(2)),
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
                .trim()
            );
        }

        #[test]
        fn format_statement_with_one_blank_line() {
            assert_eq!(
                format_block(&Block::new(
                    vec![Statement::new(
                        None,
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        line_position(1)
                    )],
                    None::new(line_position(3)),
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
                .trim()
            );
        }

        #[test]
        fn format_statement_with_two_blank_lines() {
            assert_eq!(
                format_block(&Block::new(
                    vec![Statement::new(
                        None,
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        line_position(1)
                    )],
                    None::new(line_position(4)),
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
                .trim()
            );
        }

        #[test]
        fn format_statement_with_trimmed_blank_line() {
            assert_eq!(
                format_module(&Module::new(
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
                                    Call::new(
                                        Variable::new("f", Position::fake()),
                                        vec![],
                                        Position::fake()
                                    ),
                                    line_position(1)
                                )],
                                None::new(line_position(3)),
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
    }

    mod expression {
        use super::*;

        mod call {
            use super::*;

            #[test]
            fn format() {
                assert_eq!(
                    format_expression(
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
                    format_expression(
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
        }

        mod if_ {
            use super::*;

            #[test]
            fn format_single_line() {
                assert_eq!(
                    format_expression(
                        &If::new(
                            vec![IfBranch::new(
                                Boolean::new(true, Position::fake()),
                                Block::new(vec![], None::new(Position::fake()), Position::fake())
                            )],
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
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
                    format_expression(
                        &If::new(
                            vec![IfBranch::new(
                                Boolean::new(true, Position::fake()),
                                Block::new(vec![], None::new(line_position(2)), Position::fake())
                            )],
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
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
                    format_expression(
                        &If::new(
                            vec![
                                IfBranch::new(
                                    Boolean::new(true, Position::fake()),
                                    Block::new(
                                        vec![],
                                        None::new(Position::fake()),
                                        Position::fake()
                                    )
                                ),
                                IfBranch::new(
                                    Boolean::new(false, Position::fake()),
                                    Block::new(
                                        vec![],
                                        None::new(Position::fake()),
                                        Position::fake()
                                    )
                                )
                            ],
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
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
                format_expression(
                    &IfList::new(
                        Variable::new("ys", Position::fake()),
                        "x",
                        "xs",
                        Block::new(
                            vec![],
                            Variable::new("x", Position::fake()),
                            Position::fake()
                        ),
                        Block::new(vec![], None::new(Position::fake()), Position::fake()),
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
                format_expression(
                    &IfMap::new(
                        "x",
                        Variable::new("xs", Position::fake()),
                        Variable::new("k", Position::fake()),
                        Block::new(
                            vec![],
                            Variable::new("x", Position::fake()),
                            Position::fake()
                        ),
                        Block::new(vec![], None::new(Position::fake()), Position::fake()),
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

            #[test]
            fn format_single_line() {
                assert_eq!(
                    format_expression(
                        &IfType::new(
                            "x",
                            Variable::new("y", Position::fake()),
                            vec![
                                IfTypeBranch::new(
                                    types::None::new(Position::fake()),
                                    Block::new(
                                        vec![],
                                        None::new(Position::fake()),
                                        Position::fake()
                                    )
                                ),
                                IfTypeBranch::new(
                                    types::Number::new(Position::fake()),
                                    Block::new(
                                        vec![],
                                        None::new(Position::fake()),
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
                    format_expression(
                        &IfType::new(
                            "x",
                            Variable::new("y", Position::fake()),
                            vec![
                                IfTypeBranch::new(
                                    types::None::new(Position::fake()),
                                    Block::new(
                                        vec![],
                                        None::new(line_position(2)),
                                        Position::fake()
                                    )
                                ),
                                IfTypeBranch::new(
                                    types::Number::new(Position::fake()),
                                    Block::new(
                                        vec![],
                                        None::new(Position::fake()),
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
                    format_expression(
                        &IfType::new(
                            "x",
                            Variable::new("y", Position::fake()),
                            vec![
                                IfTypeBranch::new(
                                    types::None::new(Position::fake()),
                                    Block::new(
                                        vec![],
                                        None::new(Position::fake()),
                                        Position::fake()
                                    )
                                ),
                                IfTypeBranch::new(
                                    types::Number::new(Position::fake()),
                                    Block::new(
                                        vec![],
                                        None::new(Position::fake()),
                                        Position::fake()
                                    )
                                )
                            ],
                            Some(Block::new(
                                vec![],
                                None::new(Position::fake()),
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

            #[test]
            fn format() {
                assert_eq!(
                    format_expression(
                        &Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
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
                    format_expression(
                        &Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Block::new(
                                vec![Statement::new(
                                    Some("x".into()),
                                    None::new(Position::fake()),
                                    Position::fake()
                                )],
                                None::new(Position::fake()),
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
                    format_expression(
                        &Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Block::new(vec![], None::new(line_position(2)), Position::fake()),
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
                    format_expression(
                        &Lambda::new(
                            vec![Argument::new("x", types::None::new(line_position(2)))],
                            types::None::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
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
                    format_expression(
                        &Lambda::new(
                            vec![
                                Argument::new("x", types::None::new(line_position(2))),
                                Argument::new("y", types::None::new(Position::fake()))
                            ],
                            types::None::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
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
        }

        mod number {
            use super::*;

            #[test]
            fn format_decimal_float() {
                assert_eq!(
                    format_expression(
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
                    format_expression(
                        &Number::new(NumberRepresentation::Binary("01".into()), Position::fake())
                            .into()
                    ),
                    "0b01"
                );
            }

            #[test]
            fn format_hexadecimal() {
                assert_eq!(
                    format_expression(
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
        fn format_spawn_operation() {
            assert_eq!(
                format_expression(
                    &SpawnOperation::new(
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake()
                    )
                    .into()
                ),
                "go \\() none { none }"
            );
        }

        #[test]
        fn format_string() {
            assert_eq!(
                format_expression(&ByteString::new("foo", Position::fake()).into()),
                "\"foo\""
            );
        }

        mod binary_operation {
            use super::*;

            #[test]
            fn format() {
                assert_eq!(
                    format_expression(
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
                    format_expression(
                        &BinaryOperation::new(
                            BinaryOperator::Add,
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake()
                            ),
                            Number::new(
                                NumberRepresentation::FloatingPoint("2".into()),
                                line_position(1)
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
                    format_expression(
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
                    format_expression(
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

            #[test]
            fn format_not_operation() {
                assert_eq!(
                    format_expression(
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
                    format_expression(
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
                    format_expression(
                        &UnaryOperation::new(
                            UnaryOperator::Not,
                            BinaryOperation::new(
                                BinaryOperator::And,
                                Boolean::new(true, Position::fake()),
                                Boolean::new(false, Position::fake()),
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
                format_expression(
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

            #[test]
            fn format_empty() {
                assert_eq!(
                    format_expression(
                        &List::new(types::None::new(Position::fake()), vec![], Position::fake())
                            .into()
                    ),
                    "[none]"
                );
            }

            #[test]
            fn format_element() {
                assert_eq!(
                    format_expression(
                        &List::new(
                            types::None::new(Position::fake()),
                            vec![ListElement::Single(None::new(Position::fake()).into())],
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
                    format_expression(
                        &List::new(
                            types::None::new(Position::fake()),
                            vec![
                                ListElement::Single(None::new(Position::fake()).into()),
                                ListElement::Single(None::new(Position::fake()).into())
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
                    format_expression(
                        &List::new(
                            types::None::new(Position::fake()),
                            vec![ListElement::Single(None::new(line_position(2)).into())],
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
                    format_expression(
                        &List::new(
                            types::Number::new(Position::fake()),
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

            #[test]
            fn format_comprehension() {
                assert_eq!(
                    format_expression(
                        &ListComprehension::new(
                            types::None::new(Position::fake()),
                            None::new(Position::fake()),
                            "x",
                            Variable::new("xs", Position::fake()),
                            Position::fake()
                        )
                        .into()
                    ),
                    "[none none for x in xs]"
                );
            }

            #[test]
            fn format_multi_line_comprehension() {
                assert_eq!(
                    format_expression(
                        &ListComprehension::new(
                            types::None::new(Position::fake()),
                            None::new(line_position(2)),
                            "x",
                            Variable::new("xs", Position::fake()),
                            line_position(1)
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
        }

        mod map {
            use super::*;

            #[test]
            fn format_empty() {
                assert_eq!(
                    format_expression(
                        &Map::new(
                            types::ByteString::new(Position::fake()),
                            types::Number::new(Position::fake()),
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
                    format_expression(
                        &Map::new(
                            types::ByteString::new(Position::fake()),
                            types::Number::new(Position::fake()),
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
                    format_expression(
                        &Map::new(
                            types::ByteString::new(Position::fake()),
                            types::Number::new(Position::fake()),
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
            fn format_removal() {
                assert_eq!(
                    format_expression(
                        &Map::new(
                            types::ByteString::new(Position::fake()),
                            types::Number::new(Position::fake()),
                            vec![MapElement::Removal(
                                ByteString::new("foo", Position::fake()).into()
                            )],
                            Position::fake()
                        )
                        .into()
                    ),
                    "{string: number \"foo\"}"
                );
            }

            #[test]
            fn format_map() {
                assert_eq!(
                    format_expression(
                        &Map::new(
                            types::ByteString::new(Position::fake()),
                            types::Number::new(Position::fake()),
                            vec![MapElement::Map(
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
                    format_expression(
                        &Map::new(
                            types::ByteString::new(Position::fake()),
                            types::Number::new(Position::fake()),
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
                    format_expression(
                        &Map::new(
                            types::ByteString::new(Position::fake()),
                            types::Number::new(Position::fake()),
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

            #[test]
            fn format_empty() {
                assert_eq!(
                    format_expression(&Record::new("foo", None, vec![], Position::fake()).into()),
                    "foo{}"
                );
            }

            #[test]
            fn format_field() {
                assert_eq!(
                    format_expression(
                        &Record::new(
                            "foo",
                            None,
                            vec![RecordField::new(
                                "x",
                                None::new(Position::fake()),
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
                    format_expression(
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
                    format_expression(
                        &Record::new(
                            "foo",
                            Some(Variable::new("r", Position::fake()).into()),
                            vec![RecordField::new(
                                "x",
                                None::new(Position::fake()),
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
            fn format_multi_line() {
                assert_eq!(
                    format_expression(
                        &Record::new(
                            "foo",
                            None,
                            vec![RecordField::new(
                                "x",
                                None::new(Position::fake()),
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
                    format_expression(
                        &Record::new(
                            "foo",
                            None,
                            vec![
                                RecordField::new(
                                    "x",
                                    None::new(Position::fake()),
                                    line_position(2)
                                ),
                                RecordField::new(
                                    "y",
                                    None::new(Position::fake()),
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
        }
    }
}
