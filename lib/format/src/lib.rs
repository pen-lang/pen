mod comment;
mod ir;

use ast::{types::Type, *};
use ir::{build::*, count_lines, Document};
use itertools::Itertools;
use position::Position;

pub fn format(module: &Module, comments: &[Comment]) -> String {
    let comments = comment::sort(comments);

    let (external_imports, internal_imports) = module
        .imports()
        .iter()
        .partition::<Vec<_>, _>(|import| matches!(import.module_path(), ModulePath::External(_)));

    let (external_imports, comments) = format_imports(&external_imports, &comments);
    let (internal_imports, mut comments) = format_imports(&internal_imports, comments);
    // let (foreign_imports, mut comments) =
    //     format_foreign_imports(module.foreign_imports(), comments);

    // let mut sections = vec![external_imports, internal_imports, foreign_imports];
    let mut sections = vec![
        if count_lines(&external_imports) == 0 {
            empty()
        } else {
            sequence([external_imports, line()])
        },
        if count_lines(&internal_imports) == 0 {
            empty()
        } else {
            sequence([internal_imports, line()])
        },
    ];

    for definition in module.type_definitions() {
        let (definition, new_comments) = compile_type_definition(definition, comments);

        sections.push(definition);
        comments = new_comments;
    }

    // for definition in module.definitions() {
    //     let (definition, new_comments) = format_definition(definition, comments);

    //     sections.push(definition);
    //     comments = new_comments;
    // }

    ir::format(&sections.into()).trim().to_owned() + "\n"
}

fn format_imports<'c>(
    imports: &[&Import],
    mut comments: &'c [Comment],
) -> (Document, &'c [Comment]) {
    let mut documents = vec![];

    for import in imports.iter().sorted_by_key(|import| import.module_path()) {
        let (import, new_comments) = format_import(import, comments);

        documents.push(import);
        comments = new_comments;
    }

    (documents.into(), comments)
}

fn format_import<'c>(import: &Import, comments: &'c [Comment]) -> (Document, &'c [Comment]) {
    let (block_comment, comments) = compile_block_comment(comments, import.position());

    (
        sequence([
            block_comment,
            "import ".into(),
            format_module_path(import.module_path()),
            if let Some(prefix) = import.prefix() {
                vec![" as ".into(), prefix.into()].into()
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
                            .map(|name| name.clone())
                            .intersperse(", ".into()),
                    ),
                    " }".into(),
                ])
            },
            line(),
        ]),
        comments,
    )
}

fn format_module_path(path: &ModulePath) -> Document {
    match path {
        ModulePath::External(path) => sequence([
            path.package().into(),
            "'".into(),
            format_module_path_components(path.components()),
        ]),
        ModulePath::Internal(path) => {
            sequence(["'".into(), format_module_path_components(path.components())])
        }
    }
}

fn format_module_path_components(components: &[String]) -> Document {
    components.join("'").into()
}

// fn format_foreign_imports<'c>(
//     imports: &[ForeignImport],
//     mut comments: &'c [Comment],
// ) -> (String, &'c [Comment]) {
//     let mut strings = vec![];

//     for import in imports {
//         let (string, new_comments) = format_foreign_import(import, comments);

//         strings.push(string);
//         comments = new_comments;
//     }

//     (strings.join("\n"), comments)
// }

// fn format_foreign_import<'c>(
//     import: &ForeignImport,
//     comments: &'c [Comment],
// ) -> (String, &'c [Comment]) {
//     let (block_comment, comments) = format_block_comment(comments, import.position());

//     (
//         block_comment
//             + &["import foreign".into()]
//                 .into_iter()
//                 .chain(match import.calling_convention() {
//                     CallingConvention::C => Some("\"c\"".into()),
//                     CallingConvention::Native => None,
//                 })
//                 .chain([import.name().into(), format_type(import.type_())])
//                 .collect::<Vec<_>>()
//                 .join(" "),
//         comments,
//     )
// }

fn compile_type_definition<'c>(
    definition: &TypeDefinition,
    comments: &'c [Comment],
) -> (Document, &'c [Comment]) {
    match definition {
        TypeDefinition::RecordDefinition(definition) => {
            compile_record_definition(definition, comments)
        }
        TypeDefinition::TypeAlias(alias) => compile_type_alias(alias, comments),
    }
}

fn compile_record_definition<'c>(
    definition: &RecordDefinition,
    comments: &'c [Comment],
) -> (Document, &'c [Comment]) {
    let (block_comment, comments) = compile_block_comment(comments, definition.position());
    let document = vec![
        "type ".into(),
        definition.name().into(),
        " {".into(),
        indent(
            definition
                .fields()
                .iter()
                .map(|field| {
                    vec![
                        line(),
                        field.name().into(),
                        " ".into(),
                        compile_type(field.type_()),
                    ]
                    .into()
                })
                .collect::<Vec<_>>(),
        ),
        soft_line(),
        "}".into(),
    ];

    (
        vec![
            block_comment,
            if definition.fields().is_empty() {
                flatten(document)
            } else {
                document.into()
            },
            line(),
        ]
        .into(),
        comments,
    )
}

fn compile_type_alias<'c>(alias: &TypeAlias, comments: &'c [Comment]) -> (Document, &'c [Comment]) {
    let (block_comment, comments) = compile_block_comment(comments, alias.position());

    (
        vec![
            block_comment,
            vec![
                "type ".into(),
                alias.name().into(),
                " = ".into(),
                compile_type(alias.type_()),
            ]
            .into(),
            line(),
        ]
        .into(),
        comments,
    )
}

// fn format_definition<'c>(
//     definition: &Definition,
//     comments: &'c [Comment],
// ) -> (String, &'c [Comment]) {
//     let (block_comment, comments) = format_block_comment(comments, definition.position());
//     let (lambda, comments) = format_lambda(definition.lambda(), comments);

//     (
//         block_comment
//             + &definition
//                 .foreign_export()
//                 .map(|export| {
//                     ["foreign"]
//                         .into_iter()
//                         .chain(match export.calling_convention() {
//                             CallingConvention::C => Some("\"c\""),
//                             CallingConvention::Native => None,
//                         })
//                         .collect::<Vec<_>>()
//                         .join(" ")
//                 })
//                 .into_iter()
//                 .chain([definition.name().into(), "=".into(), lambda])
//                 .collect::<Vec<_>>()
//                 .join(" "),
//         comments,
//     )
// }

fn compile_type(type_: &Type) -> Document {
    match type_ {
        Type::Any(_) => "any".into(),
        Type::Boolean(_) => "boolean".into(),
        Type::Function(function) => vec![
            "\\(".into(),
            function
                .arguments()
                .iter()
                .map(compile_type)
                .intersperse(", ".into())
                .collect::<Vec<_>>()
                .into(),
            ") ".into(),
            compile_type(function.result()),
        ]
        .into(),
        Type::List(list) => vec!["[".into(), compile_type(list.element()), "]".into()].into(),
        Type::Map(map) => vec![
            "{".into(),
            compile_type(map.key()),
            ": ".into(),
            compile_type(map.value()),
            "}".into(),
        ]
        .into(),
        Type::None(_) => "none".into(),
        Type::Number(_) => "number".into(),
        Type::Record(record) => record.name().into(),
        Type::Reference(reference) => reference.name().into(),
        Type::String(_) => "string".into(),
        Type::Union(union) => {
            let lhs = compile_type(union.lhs());

            vec![
                if union.lhs().is_function() {
                    vec!["(".into(), lhs, ")".into()].into()
                } else {
                    lhs
                },
                " | ".into(),
                compile_type(union.rhs()),
            ]
            .into()
        }
    }
}

// fn format_lambda<'c>(lambda: &Lambda, mut comments: &'c [Comment]) -> (String, &'c [Comment]) {
//     let arguments = lambda
//         .arguments()
//         .iter()
//         .map(|argument| format!("{} {}", argument.name(), format_type(argument.type_())))
//         .collect::<Vec<_>>();
//     let single_line_arguments = arguments.is_empty()
//         || Some(lambda.position().line_number())
//             == lambda
//                 .arguments()
//                 .get(0)
//                 .map(|argument| argument.type_().position().line_number())
//             && arguments.iter().all(is_single_line);

//     let arguments = if single_line_arguments {
//         "(".to_owned() + &arguments.join(", ") + ")"
//     } else {
//         let mut argument_lines = vec![];

//         for (string, argument) in arguments.into_iter().zip(lambda.arguments()) {
//             // TODO Use Argument::position().
//             let position = argument.type_().position();
//             let (block_comment, new_comments) = format_block_comment(comments, position);
//             let (suffix_comment, new_comments) = format_suffix_comment(new_comments, position);

//             argument_lines.push(indent(block_comment + &string) + "," + &suffix_comment + "\n");
//             comments = new_comments;
//         }

//         ["(".into(), "\n".into()]
//             .into_iter()
//             .chain(argument_lines)
//             .chain([")".into()])
//             .collect::<Vec<_>>()
//             .concat()
//     };

//     let (body, comments) = if single_line_arguments
//         && lambda.position().line_number() == lambda.body().expression().position().line_number()
//     {
//         format_block(lambda.body(), comments)
//     } else {
//         format_multi_line_block(lambda.body(), comments)
//     };

//     (
//         format!(
//             "\\{} {} {}",
//             arguments,
//             format_type(lambda.result_type()),
//             body
//         ),
//         comments,
//     )
// }

// fn format_block<'c>(block: &Block, comments: &'c [Comment]) -> (String, &'c [Comment]) {
//     let (expression, new_comments) = format_expression(block.expression(), comments);

//     if block.statements().is_empty() && is_single_line(&expression) {
//         (["{", &expression, "}"].join(" "), new_comments)
//     } else {
//         format_multi_line_block(block, comments)
//     }
// }

// fn format_multi_line_block<'c>(
//     block: &Block,
//     mut comments: &'c [Comment],
// ) -> (String, &'c [Comment]) {
//     let mut statements = vec![];

//     for (statement, next_position) in block.statements().iter().zip(
//         block
//             .statements()
//             .iter()
//             .skip(1)
//             .map(|statement| statement.position())
//             .chain([block.expression().position()]),
//     ) {
//         let (block_comment, new_comments) = format_block_comment(comments, statement.position());
//         // TODO Use end positions of spans when they are available.
//         let line_count = next_position.line_number() as isize
//             - statement.position().line_number() as isize
//             - comment::split_before(new_comments, next_position.line_number())
//                 .0
//                 .len() as isize;
//         let (statement_string, new_comments) = format_statement(statement, new_comments);
//         let (suffix_comment, new_comments) =
//             format_suffix_comment(new_comments, statement.position());

//         statements.push(indent(
//             block_comment
//                 + &statement_string
//                 + &suffix_comment
//                 + if count_lines(&statement_string) as isize >= line_count {
//                     ""
//                 } else {
//                     "\n"
//                 },
//         ));
//         comments = new_comments;
//     }

//     let (block_comment, comments) = format_block_comment(comments, block.expression().position());
//     let (expression, comments) = format_expression(block.expression(), comments);

//     (
//         ["{".into()]
//             .into_iter()
//             .chain(statements)
//             .chain([indent(block_comment + &expression)])
//             .chain(["}".into()])
//             .collect::<Vec<_>>()
//             .join("\n"),
//         comments,
//     )
// }

// fn format_statement<'c>(statement: &Statement, comments: &'c [Comment]) -> (String, &'c [Comment]) {
//     let (expression, comments) = format_expression(statement.expression(), comments);

//     (
//         statement
//             .name()
//             .map(|name| format!("{} =", name))
//             .into_iter()
//             .chain([expression])
//             .collect::<Vec<_>>()
//             .join(" "),
//         comments,
//     )
// }

// fn format_expression<'c>(
//     expression: &Expression,
//     mut comments: &'c [Comment],
// ) -> (String, &'c [Comment]) {
//     match expression {
//         Expression::BinaryOperation(operation) => format_binary_operation(operation, comments),
//         Expression::Call(call) => {
//             let (function, mut comments) = format_expression(call.function(), comments);
//             let head = format!("{}(", function);
//             let mut arguments = vec![];

//             for argument in call.arguments() {
//                 let (expression, new_comments) = format_expression(argument, comments);

//                 arguments.push(expression);
//                 comments = new_comments;
//             }

//             if call.arguments().is_empty()
//                 || Some(call.function().position().line_number())
//                     == call
//                         .arguments()
//                         .get(0)
//                         .map(|expression| expression.position().line_number())
//                     && arguments.iter().all(is_single_line)
//             {
//                 (
//                     [head]
//                         .into_iter()
//                         .chain(if call.arguments().is_empty() {
//                             None
//                         } else {
//                             Some(arguments.join(", "))
//                         })
//                         .chain([")".into()])
//                         .collect::<Vec<_>>()
//                         .concat(),
//                     comments,
//                 )
//             } else {
//                 (
//                     [head]
//                         .into_iter()
//                         .chain(arguments.iter().map(|argument| indent(argument) + ","))
//                         .chain([")".into()])
//                         .collect::<Vec<_>>()
//                         .join("\n"),
//                     comments,
//                 )
//             }
//         }
//         Expression::Boolean(boolean) => (
//             if boolean.value() { "true" } else { "false" }.into(),
//             comments,
//         ),
//         Expression::If(if_) => format_if(if_, comments),
//         Expression::IfList(if_) => {
//             let (list, comments) = format_expression(if_.list(), comments);
//             let (then, comments) = format_multi_line_block(if_.then(), comments);
//             let (else_, comments) = format_multi_line_block(if_.else_(), comments);

//             (
//                 [
//                     "if".into(),
//                     format!("[{}, ...{}]", if_.first_name(), if_.rest_name()),
//                     "=".into(),
//                     list,
//                     then,
//                     "else".into(),
//                     else_,
//                 ]
//                 .join(" "),
//                 comments,
//             )
//         }
//         Expression::IfMap(if_) => {
//             let (map, comments) = format_expression(if_.map(), comments);
//             let (key, comments) = format_expression(if_.key(), comments);
//             let (then, comments) = format_multi_line_block(if_.then(), comments);
//             let (else_, comments) = format_multi_line_block(if_.else_(), comments);

//             (
//                 [
//                     "if".into(),
//                     if_.name().into(),
//                     "=".into(),
//                     format!("{}[{}]", map, key),
//                     then,
//                     "else".into(),
//                     else_,
//                 ]
//                 .join(" "),
//                 comments,
//             )
//         }
//         Expression::IfType(if_) => format_if_type(if_, comments),
//         Expression::Lambda(lambda) => format_lambda(lambda, comments),
//         Expression::List(list) => {
//             let type_ = format_type(list.type_());
//             let mut elements = vec![];

//             for element in list.elements() {
//                 let (element, new_comments) = format_list_element(element, comments);

//                 elements.push(element);
//                 comments = new_comments;
//             }

//             if elements.is_empty()
//                 || Some(list.position().line_number())
//                     == list
//                         .elements()
//                         .get(0)
//                         .map(|element| element.position().line_number())
//                     && elements.iter().all(is_single_line)
//             {
//                 (
//                     ["[".into()]
//                         .into_iter()
//                         .chain([[type_]
//                             .into_iter()
//                             .chain(if elements.is_empty() {
//                                 None
//                             } else {
//                                 Some(elements.join(", "))
//                             })
//                             .collect::<Vec<_>>()
//                             .join(" ")])
//                         .chain(["]".into()])
//                         .collect::<Vec<_>>()
//                         .concat(),
//                     comments,
//                 )
//             } else {
//                 (
//                     [format!("[{}", type_)]
//                         .into_iter()
//                         .chain(elements.into_iter().map(|element| indent(element) + ","))
//                         .chain(["]".into()])
//                         .collect::<Vec<_>>()
//                         .join("\n"),
//                     comments,
//                 )
//             }
//         }
//         Expression::ListComprehension(comprehension) => {
//             let type_ = format_type(comprehension.type_());
//             let (element, comments) = format_expression(comprehension.element(), comments);
//             let (list, comments) = format_expression(comprehension.list(), comments);

//             (
//                 if comprehension.position().line_number()
//                     == comprehension.element().position().line_number()
//                     && is_single_line(&element)
//                     && is_single_line(&list)
//                 {
//                     ["[".into()]
//                         .into_iter()
//                         .chain([[
//                             type_,
//                             element,
//                             "for".into(),
//                             comprehension.element_name().into(),
//                             "in".into(),
//                             list,
//                         ]
//                         .join(" ")])
//                         .chain(["]".into()])
//                         .collect::<Vec<_>>()
//                         .concat()
//                 } else {
//                     [format!("[{}", type_)]
//                         .into_iter()
//                         .chain(
//                             [
//                                 element,
//                                 format!("for {} in {}", comprehension.element_name(), list),
//                             ]
//                             .iter()
//                             .map(indent),
//                         )
//                         .chain(["]".into()])
//                         .collect::<Vec<_>>()
//                         .join("\n")
//                 },
//                 comments,
//             )
//         }
//         Expression::Map(map) => {
//             let type_ = format_type(map.key_type()) + ": " + &format_type(map.value_type());
//             let mut elements = vec![];

//             for element in map.elements() {
//                 let (element, new_comments) = format_map_element(element, comments);

//                 elements.push(element);
//                 comments = new_comments;
//             }

//             (
//                 if elements.is_empty()
//                     || Some(map.position().line_number())
//                         == map
//                             .elements()
//                             .get(0)
//                             .map(|element| element.position().line_number())
//                         && elements.iter().all(is_single_line)
//                 {
//                     ["{".into()]
//                         .into_iter()
//                         .chain([[type_]
//                             .into_iter()
//                             .chain(if map.elements().is_empty() {
//                                 None
//                             } else {
//                                 Some(elements.join(", "))
//                             })
//                             .collect::<Vec<_>>()
//                             .join(" ")])
//                         .chain(["}".into()])
//                         .collect::<Vec<_>>()
//                         .concat()
//                 } else {
//                     [format!("{{{}", type_)]
//                         .into_iter()
//                         .chain(elements.into_iter().map(|element| indent(element) + ","))
//                         .chain(["}".into()])
//                         .collect::<Vec<_>>()
//                         .join("\n")
//                 },
//                 comments,
//             )
//         }
//         Expression::None(_) => ("none".into(), comments),
//         Expression::Number(number) => (
//             match number.value() {
//                 NumberRepresentation::Binary(string) => "0b".to_owned() + string,
//                 NumberRepresentation::Hexadecimal(string) => {
//                     "0x".to_owned() + &string.to_uppercase()
//                 }
//                 NumberRepresentation::FloatingPoint(string) => string.clone(),
//             },
//             comments,
//         ),
//         Expression::Record(record) => {
//             let (old_record, mut comments) = if let Some(record) = record.record() {
//                 let (record, comments) = format_expression(record, comments);

//                 (Some(format!("...{}", record)), comments)
//             } else {
//                 (None, comments)
//             };
//             let mut elements = old_record.into_iter().collect::<Vec<_>>();

//             for field in record.fields() {
//                 let (expression, new_comments) = format_expression(field.expression(), comments);

//                 elements.push(format!("{}: {}", field.name(), expression,));
//                 comments = new_comments;
//             }

//             (
//                 if record.fields().is_empty()
//                     || Some(record.position().line_number())
//                         == record
//                             .fields()
//                             .get(0)
//                             .map(|field| field.position().line_number())
//                         && elements.iter().all(is_single_line)
//                 {
//                     [record.type_name().into(), "{".into()]
//                         .into_iter()
//                         .chain(if record.record().is_none() && record.fields().is_empty() {
//                             None
//                         } else {
//                             Some(elements.join(", "))
//                         })
//                         .chain(["}".into()])
//                         .collect::<Vec<_>>()
//                         .concat()
//                 } else {
//                     [record.type_name().to_owned() + "{"]
//                         .into_iter()
//                         .chain(elements.into_iter().map(|line| indent(line) + ","))
//                         .chain(["}".into()])
//                         .collect::<Vec<_>>()
//                         .join("\n")
//                 },
//                 comments,
//             )
//         }
//         Expression::RecordDeconstruction(deconstruction) => {
//             let (record, comments) = format_expression(deconstruction.expression(), comments);

//             (format!("{}.{}", record, deconstruction.name()), comments)
//         }
//         Expression::SpawnOperation(operation) => {
//             let (lambda, comments) = format_lambda(operation.function(), comments);

//             (format!("go {}", lambda), comments)
//         }
//         Expression::String(string) => (format!("\"{}\"", string.value()), comments),
//         Expression::UnaryOperation(operation) => {
//             let (operand, comments) = format_expression(operation.expression(), comments);
//             let operand = if matches!(operation.expression(), Expression::BinaryOperation(_)) {
//                 "(".to_owned() + &operand + ")"
//             } else {
//                 operand
//             };

//             (
//                 match operation.operator() {
//                     UnaryOperator::Not => "!".to_owned() + &operand,
//                     UnaryOperator::Try => operand + "?",
//                 },
//                 comments,
//             )
//         }
//         Expression::Variable(variable) => (variable.name().into(), comments),
//     }
// }

// fn format_if<'c>(if_: &If, original_comments: &'c [Comment]) -> (String, &'c [Comment]) {
//     let (branches, comments) = {
//         let mut branches = vec![];
//         let mut comments = original_comments;

//         for branch in if_.branches() {
//             let (expression, new_comments) = format_expression(branch.condition(), comments);
//             let (block, new_comments) = format_block(branch.block(), new_comments);

//             branches.push(expression + " " + &block);
//             comments = new_comments;
//         }

//         (branches, comments)
//     };
//     let (else_, comments) = format_block(if_.else_(), comments);

//     if branches.len() == 1
//         && branches.iter().chain([&else_]).all(is_single_line)
//         && Some(if_.position().line_number())
//             == if_
//                 .branches()
//                 .get(0)
//                 .map(|branch| branch.block().expression().position().line_number())
//     {
//         (
//             branches
//                 .into_iter()
//                 .flat_map(|branch| ["if".into(), branch, "else".into()])
//                 .chain([else_])
//                 .collect::<Vec<_>>()
//                 .join(" "),
//             comments,
//         )
//     } else {
//         let mut parts = vec![];
//         let mut comments = original_comments;

//         for branch in if_.branches() {
//             let (expression, new_comments) = format_expression(branch.condition(), comments);
//             let (block, new_comments) = format_multi_line_block(branch.block(), new_comments);

//             parts.extend(["if".into(), expression, block, "else".into()]);
//             comments = new_comments;
//         }

//         let (else_, comments) = format_multi_line_block(if_.else_(), comments);

//         parts.push(else_);

//         (parts.join(" "), comments)
//     }
// }

// fn format_if_type<'c>(if_: &IfType, original_comments: &'c [Comment]) -> (String, &'c [Comment]) {
//     let (argument, original_comments) = format_expression(if_.argument(), original_comments);
//     let (branches, comments) = {
//         let mut branches = vec![];
//         let mut comments = original_comments;

//         for branch in if_.branches() {
//             let (block, new_comments) = format_block(branch.block(), comments);

//             branches.push(format_type(branch.type_()) + " " + &block);
//             comments = new_comments;
//         }

//         (branches, comments)
//     };
//     let (else_, comments) = if let Some(block) = if_.else_() {
//         let (block, comments) = format_block(block, comments);

//         (Some(block), comments)
//     } else {
//         (None, comments)
//     };

//     let head = [
//         "if".into(),
//         if_.name().into(),
//         "=".into(),
//         argument.clone(),
//         "as".into(),
//     ];

//     if branches.iter().chain(&else_).count() <= 2
//         && [&argument]
//             .into_iter()
//             .chain(&branches)
//             .chain(else_.as_ref())
//             .all(is_single_line)
//         && Some(if_.position().line_number())
//             == if_
//                 .branches()
//                 .get(0)
//                 .map(|branch| branch.block().expression().position().line_number())
//     {
//         (
//             head.into_iter()
//                 .chain([branches.join(" else if ")])
//                 .chain(else_.into_iter().flat_map(|block| ["else".into(), block]))
//                 .collect::<Vec<_>>()
//                 .join(" "),
//             comments,
//         )
//     } else {
//         let mut branches = vec![];
//         let mut comments = original_comments;

//         for branch in if_.branches() {
//             let (block, new_comments) = format_multi_line_block(branch.block(), comments);

//             branches.push(format_type(branch.type_()) + " " + &block);
//             comments = new_comments;
//         }

//         let (else_, comments) = if let Some(block) = if_.else_() {
//             let (block, comments) = format_multi_line_block(block, comments);

//             (Some(block), comments)
//         } else {
//             (None, comments)
//         };

//         (
//             head.into_iter()
//                 .chain([branches.join(" else if ")])
//                 .chain(else_.into_iter().flat_map(|block| ["else".into(), block]))
//                 .collect::<Vec<_>>()
//                 .join(" "),
//             comments,
//         )
//     }
// }

// fn format_list_element<'c>(
//     element: &ListElement,
//     comments: &'c [Comment],
// ) -> (String, &'c [Comment]) {
//     match element {
//         ListElement::Multiple(expression) => {
//             let (expression, comments) = format_expression(expression, comments);

//             (format!("...{}", expression), comments)
//         }
//         ListElement::Single(expression) => format_expression(expression, comments),
//     }
// }

// fn format_map_element<'c>(
//     element: &MapElement,
//     comments: &'c [Comment],
// ) -> (String, &'c [Comment]) {
//     match element {
//         MapElement::Map(expression) => {
//             let (expression, comments) = format_expression(expression, comments);

//             (format!("...{}", expression), comments)
//         }
//         MapElement::Insertion(entry) => {
//             let (key, comments) = format_expression(entry.key(), comments);
//             let (value, comments) = format_expression(entry.value(), comments);

//             (format!("{}: {}", key, value), comments)
//         }
//         MapElement::Removal(expression) => format_expression(expression, comments),
//     }
// }

// fn format_binary_operation<'c>(
//     operation: &BinaryOperation,
//     comments: &'c [Comment],
// ) -> (String, &'c [Comment]) {
//     let single_line =
//         operation.lhs().position().line_number() == operation.rhs().position().line_number();
//     let operator = format_binary_operator(operation.operator()).into();
//     let (lhs, comments) = format_operand(operation.lhs(), operation.operator(), comments);
//     let (rhs, comments) = format_operand(operation.rhs(), operation.operator(), comments);

//     (
//         [
//             lhs,
//             [
//                 if single_line {
//                     operator
//                 } else {
//                     indent(operator)
//                 },
//                 rhs,
//             ]
//             .join(" "),
//         ]
//         .join(if single_line { " " } else { "\n" }),
//         comments,
//     )
// }

// fn format_operand<'c>(
//     operand: &Expression,
//     parent_operator: BinaryOperator,
//     comments: &'c [Comment],
// ) -> (String, &'c [Comment]) {
//     let (string, comments) = format_expression(operand, comments);

//     (
//         if match operand {
//             Expression::BinaryOperation(operation) => Some(operation),
//             _ => None,
//         }
//         .map(|operand| operator_priority(operand.operator()) < operator_priority(parent_operator))
//         .unwrap_or_default()
//         {
//             format!("({})", string)
//         } else {
//             string
//         },
//         comments,
//     )
// }

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

// TODO Define this in an ast crate?
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

fn compile_suffix_comment<'c>(
    comments: &'c [Comment],
    position: &Position,
) -> (Document, &'c [Comment]) {
    let (comment, comments) = comment::split_current(comments, position.line_number());

    (
        comment
            .map(|comment| line_suffix(" #".to_owned() + comment.line().trim_end()))
            .unwrap_or_else(|| empty()),
        comments,
    )
}

fn compile_block_comment<'c>(
    comments: &'c [Comment],
    position: &Position,
) -> (Document, &'c [Comment]) {
    let (before, after) = comment::split_before(comments, position.line_number());

    (
        compile_all_comments(before, Some(position.line_number())),
        after,
    )
}

fn compile_all_comments(comments: &[Comment], last_line_number: Option<usize>) -> Document {
    comments
        .iter()
        .zip(
            comments
                .iter()
                .skip(1)
                .map(|comment| comment.position().line_number())
                .chain([last_line_number.unwrap_or(usize::MAX)]),
        )
        .map(|(comment, next_line_number)| {
            vec![
                "#".into(),
                comment.line().trim_end().into(),
                if comment.position().line_number() + 1 < next_line_number {
                    line()
                } else {
                    empty()
                },
            ]
            .into()
        })
        .collect::<Vec<_>>()
        .into()
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
        fn format_unqualified_module_import() {
            assert_eq!(
                format_module(&Module::new(
                    vec![Import::new(
                        InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                        None,
                        vec!["Baz".into(), "Blah".into()],
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

    // #[test]
    // fn format_foreign_import() {
    //     assert_eq!(
    //         format_module(&Module::new(
    //             vec![],
    //             vec![ForeignImport::new(
    //                 "foo",
    //                 CallingConvention::Native,
    //                 types::Function::new(
    //                     vec![],
    //                     types::None::new(Position::fake()),
    //                     Position::fake()
    //                 ),
    //                 Position::fake(),
    //             )],
    //             vec![],
    //             vec![],
    //             Position::fake()
    //         )),
    //         "import foreign foo \\() none\n"
    //     );
    // }

    // #[test]
    // fn format_foreign_import_with_c_calling_convention() {
    //     assert_eq!(
    //         format_module(&Module::new(
    //             vec![],
    //             vec![ForeignImport::new(
    //                 "foo",
    //                 CallingConvention::C,
    //                 types::Function::new(
    //                     vec![],
    //                     types::None::new(Position::fake()),
    //                     Position::fake()
    //                 ),
    //                 Position::fake(),
    //             )],
    //             vec![],
    //             vec![],
    //             Position::fake()
    //         )),
    //         "import foreign \"c\" foo \\() none\n"
    //     );
    // }

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

    mod type_ {
        use super::*;

        fn format_type(type_: &Type) -> String {
            ir::format(&compile_type(type_))
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
    }

    // mod definition {
    //     use super::*;

    //     #[test]
    //     fn format_with_no_argument_and_no_statement() {
    //         assert_eq!(
    //             format_module(&Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![],
    //                 vec![Definition::new(
    //                     "foo",
    //                     Lambda::new(
    //                         vec![],
    //                         types::None::new(Position::fake()),
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         Position::fake(),
    //                     ),
    //                     None,
    //                     Position::fake()
    //                 )],
    //                 Position::fake()
    //             )),
    //             "foo = \\() none { none }\n"
    //         );
    //     }

    //     #[test]
    //     fn format_with_argument() {
    //         assert_eq!(
    //             format_module(&Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![],
    //                 vec![Definition::new(
    //                     "foo",
    //                     Lambda::new(
    //                         vec![Argument::new("x", types::None::new(Position::fake()))],
    //                         types::None::new(Position::fake()),
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         Position::fake(),
    //                     ),
    //                     None,
    //                     Position::fake()
    //                 )],
    //                 Position::fake()
    //             )),
    //             "foo = \\(x none) none { none }\n"
    //         );
    //     }

    //     #[test]
    //     fn format_with_statement() {
    //         assert_eq!(
    //             format_module(&Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![],
    //                 vec![Definition::new(
    //                     "foo",
    //                     Lambda::new(
    //                         vec![],
    //                         types::None::new(Position::fake()),
    //                         Block::new(
    //                             vec![Statement::new(
    //                                 None,
    //                                 None::new(Position::fake()),
    //                                 Position::fake()
    //                             )],
    //                             None::new(Position::fake()),
    //                             Position::fake()
    //                         ),
    //                         Position::fake(),
    //                     ),
    //                     None,
    //                     Position::fake()
    //                 )],
    //                 Position::fake()
    //             )),
    //             indoc!(
    //                 "
    //             foo = \\() none {
    //               none
    //               none
    //             }
    //             "
    //             )
    //         );
    //     }

    //     #[test]
    //     fn format_returning_lambda() {
    //         assert_eq!(
    //             format_module(&Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![],
    //                 vec![Definition::new(
    //                     "foo",
    //                     Lambda::new(
    //                         vec![],
    //                         types::Function::new(
    //                             vec![],
    //                             types::None::new(Position::fake()),
    //                             Position::fake()
    //                         ),
    //                         Block::new(
    //                             vec![],
    //                             Lambda::new(
    //                                 vec![],
    //                                 types::None::new(Position::fake()),
    //                                 Block::new(
    //                                     vec![],
    //                                     None::new(Position::fake()),
    //                                     Position::fake()
    //                                 ),
    //                                 Position::fake(),
    //                             ),
    //                             Position::fake()
    //                         ),
    //                         Position::fake(),
    //                     ),
    //                     None,
    //                     Position::fake()
    //                 )],
    //                 Position::fake()
    //             )),
    //             "foo = \\() \\() none { \\() none { none } }\n"
    //         );
    //     }

    //     #[test]
    //     fn format_with_foreign_export() {
    //         assert_eq!(
    //             format_module(&Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![],
    //                 vec![Definition::new(
    //                     "foo",
    //                     Lambda::new(
    //                         vec![],
    //                         types::None::new(Position::fake()),
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         Position::fake(),
    //                     ),
    //                     Some(ForeignExport::new(CallingConvention::Native)),
    //                     Position::fake()
    //                 )],
    //                 Position::fake()
    //             )),
    //             "foreign foo = \\() none { none }\n"
    //         );
    //     }

    //     #[test]
    //     fn format_with_foreign_export_and_custom_calling_convention() {
    //         assert_eq!(
    //             format_module(&Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![],
    //                 vec![Definition::new(
    //                     "foo",
    //                     Lambda::new(
    //                         vec![],
    //                         types::None::new(Position::fake()),
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         Position::fake(),
    //                     ),
    //                     Some(ForeignExport::new(CallingConvention::C)),
    //                     Position::fake()
    //                 )],
    //                 Position::fake()
    //             )),
    //             "foreign \"c\" foo = \\() none { none }\n"
    //         );
    //     }
    // }

    // mod block {
    //     use super::*;

    //     fn format(block: &Block) -> String {
    //         format_block(block, &[]).0
    //     }

    //     #[test]
    //     fn format_() {
    //         assert_eq!(
    //             format(&Block::new(
    //                 vec![],
    //                 None::new(Position::fake()),
    //                 Position::fake()
    //             )),
    //             "{ none }"
    //         );
    //     }

    //     #[test]
    //     fn format_statement() {
    //         assert_eq!(
    //             format(&Block::new(
    //                 vec![Statement::new(
    //                     None,
    //                     Call::new(
    //                         Variable::new("f", Position::fake()),
    //                         vec![],
    //                         Position::fake()
    //                     ),
    //                     Position::fake()
    //                 )],
    //                 None::new(Position::fake()),
    //                 Position::fake()
    //             )),
    //             indoc!(
    //                 "
    //                 {
    //                   f()
    //                   none
    //                 }
    //                 "
    //             )
    //             .trim()
    //         );
    //     }

    //     #[test]
    //     fn format_statement_with_name() {
    //         assert_eq!(
    //             format(&Block::new(
    //                 vec![Statement::new(
    //                     Some("x".into()),
    //                     Call::new(
    //                         Variable::new("f", Position::fake()),
    //                         vec![],
    //                         Position::fake()
    //                     ),
    //                     Position::fake()
    //                 )],
    //                 None::new(Position::fake()),
    //                 Position::fake()
    //             )),
    //             indoc!(
    //                 "
    //                 {
    //                   x = f()
    //                   none
    //                 }
    //                 "
    //             )
    //             .trim()
    //         );
    //     }

    //     #[test]
    //     fn format_statement_with_no_blank_line() {
    //         assert_eq!(
    //             format(&Block::new(
    //                 vec![Statement::new(
    //                     None,
    //                     Call::new(
    //                         Variable::new("f", Position::fake()),
    //                         vec![],
    //                         Position::fake()
    //                     ),
    //                     line_position(1)
    //                 )],
    //                 None::new(line_position(2)),
    //                 Position::fake()
    //             )),
    //             indoc!(
    //                 "
    //                 {
    //                   f()
    //                   none
    //                 }
    //                 "
    //             )
    //             .trim()
    //         );
    //     }

    //     #[test]
    //     fn format_statement_with_one_blank_line() {
    //         assert_eq!(
    //             format(&Block::new(
    //                 vec![Statement::new(
    //                     None,
    //                     Call::new(
    //                         Variable::new("f", Position::fake()),
    //                         vec![],
    //                         Position::fake()
    //                     ),
    //                     line_position(1)
    //                 )],
    //                 None::new(line_position(3)),
    //                 Position::fake()
    //             )),
    //             indoc!(
    //                 "
    //                 {
    //                   f()

    //                   none
    //                 }
    //                 "
    //             )
    //             .trim()
    //         );
    //     }

    //     #[test]
    //     fn format_statement_with_two_blank_lines() {
    //         assert_eq!(
    //             format(&Block::new(
    //                 vec![Statement::new(
    //                     None,
    //                     Call::new(
    //                         Variable::new("f", Position::fake()),
    //                         vec![],
    //                         Position::fake()
    //                     ),
    //                     line_position(1)
    //                 )],
    //                 None::new(line_position(4)),
    //                 Position::fake()
    //             )),
    //             indoc!(
    //                 "
    //                 {
    //                   f()

    //                   none
    //                 }
    //                 "
    //             )
    //             .trim()
    //         );
    //     }

    //     #[test]
    //     fn format_statement_with_trimmed_blank_line() {
    //         assert_eq!(
    //             format_module(&Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![],
    //                 vec![Definition::new(
    //                     "foo",
    //                     Lambda::new(
    //                         vec![],
    //                         types::None::new(Position::fake()),
    //                         Block::new(
    //                             vec![Statement::new(
    //                                 None,
    //                                 Call::new(
    //                                     Variable::new("f", Position::fake()),
    //                                     vec![],
    //                                     Position::fake()
    //                                 ),
    //                                 line_position(1)
    //                             )],
    //                             None::new(line_position(3)),
    //                             Position::fake()
    //                         ),
    //                         Position::fake(),
    //                     ),
    //                     None,
    //                     Position::fake()
    //                 )],
    //                 Position::fake()
    //             )),
    //             indoc!(
    //                 "
    //                 foo = \\() none {
    //                   f()

    //                   none
    //                 }
    //                 "
    //             )
    //         );
    //     }
    // }

    // mod expression {
    //     use super::*;

    //     fn format(expression: &Expression) -> String {
    //         format_expression(expression, &[]).0
    //     }

    //     mod call {
    //         use super::*;

    //         #[test]
    //         fn format_() {
    //             assert_eq!(
    //                 format(
    //                     &Call::new(
    //                         Variable::new("foo", Position::fake()),
    //                         vec![
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("1".into()),
    //                                 Position::fake()
    //                             )
    //                             .into(),
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("2".into()),
    //                                 Position::fake()
    //                             )
    //                             .into(),
    //                         ],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "foo(1, 2)"
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line() {
    //             assert_eq!(
    //                 format(
    //                     &Call::new(
    //                         Variable::new("foo", line_position(1)),
    //                         vec![
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("1".into()),
    //                                 line_position(2)
    //                             )
    //                             .into(),
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("2".into()),
    //                                 Position::fake()
    //                             )
    //                             .into(),
    //                         ],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     foo(
    //                       1,
    //                       2,
    //                     )
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }
    //     }

    //     mod if_ {
    //         use super::*;

    //         #[test]
    //         fn format_single_line() {
    //             assert_eq!(
    //                 format(
    //                     &If::new(
    //                         vec![IfBranch::new(
    //                             Boolean::new(true, Position::fake()),
    //                             Block::new(vec![], None::new(Position::fake()), Position::fake())
    //                         )],
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "if true { none } else { none }"
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line_with_multi_line_input() {
    //             assert_eq!(
    //                 format(
    //                     &If::new(
    //                         vec![IfBranch::new(
    //                             Boolean::new(true, Position::fake()),
    //                             Block::new(vec![], None::new(line_position(2)), Position::fake())
    //                         )],
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     if true {
    //                       none
    //                     } else {
    //                       none
    //                     }
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line_with_multiple_branches() {
    //             assert_eq!(
    //                 format(
    //                     &If::new(
    //                         vec![
    //                             IfBranch::new(
    //                                 Boolean::new(true, Position::fake()),
    //                                 Block::new(
    //                                     vec![],
    //                                     None::new(Position::fake()),
    //                                     Position::fake()
    //                                 )
    //                             ),
    //                             IfBranch::new(
    //                                 Boolean::new(false, Position::fake()),
    //                                 Block::new(
    //                                     vec![],
    //                                     None::new(Position::fake()),
    //                                     Position::fake()
    //                                 )
    //                             )
    //                         ],
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     if true {
    //                       none
    //                     } else if false {
    //                       none
    //                     } else {
    //                       none
    //                     }
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }
    //     }

    //     #[test]
    //     fn format_if_list() {
    //         assert_eq!(
    //             format(
    //                 &IfList::new(
    //                     Variable::new("ys", Position::fake()),
    //                     "x",
    //                     "xs",
    //                     Block::new(
    //                         vec![],
    //                         Variable::new("x", Position::fake()),
    //                         Position::fake()
    //                     ),
    //                     Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                     Position::fake()
    //                 )
    //                 .into()
    //             ),
    //             indoc!(
    //                 "
    //                 if [x, ...xs] = ys {
    //                   x
    //                 } else {
    //                   none
    //                 }
    //                 "
    //             )
    //             .trim()
    //         );
    //     }

    //     #[test]
    //     fn format_if_map() {
    //         assert_eq!(
    //             format(
    //                 &IfMap::new(
    //                     "x",
    //                     Variable::new("xs", Position::fake()),
    //                     Variable::new("k", Position::fake()),
    //                     Block::new(
    //                         vec![],
    //                         Variable::new("x", Position::fake()),
    //                         Position::fake()
    //                     ),
    //                     Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                     Position::fake()
    //                 )
    //                 .into()
    //             ),
    //             indoc!(
    //                 "
    //                 if x = xs[k] {
    //                   x
    //                 } else {
    //                   none
    //                 }
    //                 "
    //             )
    //             .trim()
    //         );
    //     }

    //     mod if_type {
    //         use super::*;

    //         #[test]
    //         fn format_single_line() {
    //             assert_eq!(
    //                 format(
    //                     &IfType::new(
    //                         "x",
    //                         Variable::new("y", Position::fake()),
    //                         vec![
    //                             IfTypeBranch::new(
    //                                 types::None::new(Position::fake()),
    //                                 Block::new(
    //                                     vec![],
    //                                     None::new(Position::fake()),
    //                                     Position::fake()
    //                                 )
    //                             ),
    //                             IfTypeBranch::new(
    //                                 types::Number::new(Position::fake()),
    //                                 Block::new(
    //                                     vec![],
    //                                     None::new(Position::fake()),
    //                                     Position::fake()
    //                                 )
    //                             )
    //                         ],
    //                         None,
    //                         Position::fake(),
    //                     )
    //                     .into()
    //                 ),
    //                 "if x = y as none { none } else if number { none }"
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line() {
    //             assert_eq!(
    //                 format(
    //                     &IfType::new(
    //                         "x",
    //                         Variable::new("y", Position::fake()),
    //                         vec![
    //                             IfTypeBranch::new(
    //                                 types::None::new(Position::fake()),
    //                                 Block::new(
    //                                     vec![],
    //                                     None::new(line_position(2)),
    //                                     Position::fake()
    //                                 )
    //                             ),
    //                             IfTypeBranch::new(
    //                                 types::Number::new(Position::fake()),
    //                                 Block::new(
    //                                     vec![],
    //                                     None::new(Position::fake()),
    //                                     Position::fake()
    //                                 )
    //                             )
    //                         ],
    //                         None,
    //                         line_position(1),
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     if x = y as none {
    //                       none
    //                     } else if number {
    //                       none
    //                     }
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }

    //         #[test]
    //         fn format_with_else_block() {
    //             assert_eq!(
    //                 format(
    //                     &IfType::new(
    //                         "x",
    //                         Variable::new("y", Position::fake()),
    //                         vec![
    //                             IfTypeBranch::new(
    //                                 types::None::new(Position::fake()),
    //                                 Block::new(
    //                                     vec![],
    //                                     None::new(Position::fake()),
    //                                     Position::fake()
    //                                 )
    //                             ),
    //                             IfTypeBranch::new(
    //                                 types::Number::new(Position::fake()),
    //                                 Block::new(
    //                                     vec![],
    //                                     None::new(Position::fake()),
    //                                     Position::fake()
    //                                 )
    //                             )
    //                         ],
    //                         Some(Block::new(
    //                             vec![],
    //                             None::new(Position::fake()),
    //                             Position::fake()
    //                         )),
    //                         Position::fake(),
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                 if x = y as none {
    //                   none
    //                 } else if number {
    //                   none
    //                 } else {
    //                   none
    //                 }
    //                 "
    //                 )
    //                 .trim()
    //             );
    //         }
    //     }

    //     mod lambda {
    //         use super::*;

    //         #[test]
    //         fn format_() {
    //             assert_eq!(
    //                 format(
    //                     &Lambda::new(
    //                         vec![],
    //                         types::None::new(Position::fake()),
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "\\() none { none }"
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line_body() {
    //             assert_eq!(
    //                 format(
    //                     &Lambda::new(
    //                         vec![],
    //                         types::None::new(Position::fake()),
    //                         Block::new(
    //                             vec![Statement::new(
    //                                 Some("x".into()),
    //                                 None::new(Position::fake()),
    //                                 Position::fake()
    //                             )],
    //                             None::new(Position::fake()),
    //                             Position::fake()
    //                         ),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     \\() none {
    //                       x = none
    //                       none
    //                     }
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }

    //         #[test]
    //         fn format_single_line_arguments_with_multi_line_body_of_expression() {
    //             assert_eq!(
    //                 format(
    //                     &Lambda::new(
    //                         vec![],
    //                         types::None::new(Position::fake()),
    //                         Block::new(vec![], None::new(line_position(2)), Position::fake()),
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     \\() none {
    //                       none
    //                     }
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line_argument() {
    //             assert_eq!(
    //                 format(
    //                     &Lambda::new(
    //                         vec![Argument::new("x", types::None::new(line_position(2)))],
    //                         types::None::new(Position::fake()),
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     \\(
    //                       x none,
    //                     ) none {
    //                       none
    //                     }
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line_arguments() {
    //             assert_eq!(
    //                 format(
    //                     &Lambda::new(
    //                         vec![
    //                             Argument::new("x", types::None::new(line_position(2))),
    //                             Argument::new("y", types::None::new(Position::fake()))
    //                         ],
    //                         types::None::new(Position::fake()),
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     \\(
    //                       x none,
    //                       y none,
    //                     ) none {
    //                       none
    //                     }
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }
    //     }

    //     mod number {
    //         use super::*;

    //         #[test]
    //         fn format_decimal_float() {
    //             assert_eq!(
    //                 format(
    //                     &Number::new(
    //                         NumberRepresentation::FloatingPoint("42".into()),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "42"
    //             );
    //         }

    //         #[test]
    //         fn format_binary() {
    //             assert_eq!(
    //                 format(
    //                     &Number::new(NumberRepresentation::Binary("01".into()), Position::fake())
    //                         .into()
    //                 ),
    //                 "0b01"
    //             );
    //         }

    //         #[test]
    //         fn format_hexadecimal() {
    //             assert_eq!(
    //                 format(
    //                     &Number::new(
    //                         NumberRepresentation::Hexadecimal("fa".into()),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "0xFA"
    //             );
    //         }
    //     }

    //     #[test]
    //     fn format_spawn_operation() {
    //         assert_eq!(
    //             format(
    //                 &SpawnOperation::new(
    //                     Lambda::new(
    //                         vec![],
    //                         types::None::new(Position::fake()),
    //                         Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                         Position::fake(),
    //                     ),
    //                     Position::fake()
    //                 )
    //                 .into()
    //             ),
    //             "go \\() none { none }"
    //         );
    //     }

    //     #[test]
    //     fn format_string() {
    //         assert_eq!(
    //             format(&ByteString::new("foo", Position::fake()).into()),
    //             "\"foo\""
    //         );
    //     }

    //     mod binary_operation {
    //         use super::*;

    //         #[test]
    //         fn format_() {
    //             assert_eq!(
    //                 format(
    //                     &BinaryOperation::new(
    //                         BinaryOperator::Add,
    //                         Number::new(
    //                             NumberRepresentation::FloatingPoint("1".into()),
    //                             Position::fake()
    //                         ),
    //                         Number::new(
    //                             NumberRepresentation::FloatingPoint("2".into()),
    //                             Position::fake()
    //                         ),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "1 + 2"
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line() {
    //             assert_eq!(
    //                 format(
    //                     &BinaryOperation::new(
    //                         BinaryOperator::Add,
    //                         Number::new(
    //                             NumberRepresentation::FloatingPoint("1".into()),
    //                             line_position(1)
    //                         ),
    //                         Number::new(
    //                             NumberRepresentation::FloatingPoint("2".into()),
    //                             line_position(2)
    //                         ),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     1
    //                       + 2
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }

    //         #[test]
    //         fn format_nested_operations() {
    //             assert_eq!(
    //                 format(
    //                     &BinaryOperation::new(
    //                         BinaryOperator::Add,
    //                         Number::new(
    //                             NumberRepresentation::FloatingPoint("1".into()),
    //                             Position::fake()
    //                         ),
    //                         BinaryOperation::new(
    //                             BinaryOperator::Multiply,
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("2".into()),
    //                                 Position::fake()
    //                             ),
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("3".into()),
    //                                 Position::fake()
    //                             ),
    //                             Position::fake()
    //                         ),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "1 + 2 * 3"
    //             );
    //         }

    //         #[test]
    //         fn format_nested_operations_with_priority() {
    //             assert_eq!(
    //                 format(
    //                     &BinaryOperation::new(
    //                         BinaryOperator::Multiply,
    //                         Number::new(
    //                             NumberRepresentation::FloatingPoint("1".into()),
    //                             Position::fake()
    //                         ),
    //                         BinaryOperation::new(
    //                             BinaryOperator::Add,
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("2".into()),
    //                                 Position::fake()
    //                             ),
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("3".into()),
    //                                 Position::fake()
    //                             ),
    //                             Position::fake()
    //                         ),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "1 * (2 + 3)"
    //             );
    //         }
    //     }

    //     mod unary_operation {
    //         use super::*;

    //         #[test]
    //         fn format_not_operation() {
    //             assert_eq!(
    //                 format(
    //                     &UnaryOperation::new(
    //                         UnaryOperator::Not,
    //                         Variable::new("x", Position::fake()),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "!x"
    //             );
    //         }

    //         #[test]
    //         fn format_try_operation() {
    //             assert_eq!(
    //                 format(
    //                     &UnaryOperation::new(
    //                         UnaryOperator::Try,
    //                         Variable::new("x", Position::fake()),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "x?"
    //             );
    //         }

    //         #[test]
    //         fn format_with_binary_operation() {
    //             assert_eq!(
    //                 format(
    //                     &UnaryOperation::new(
    //                         UnaryOperator::Not,
    //                         BinaryOperation::new(
    //                             BinaryOperator::And,
    //                             Boolean::new(true, Position::fake()),
    //                             Boolean::new(false, Position::fake()),
    //                             Position::fake()
    //                         ),
    //                         Position::fake()
    //                     )
    //                     .into(),
    //                 ),
    //                 "!(true & false)"
    //             );
    //         }
    //     }

    //     #[test]
    //     fn format_record_deconstruction() {
    //         assert_eq!(
    //             format(
    //                 &RecordDeconstruction::new(
    //                     Variable::new("x", Position::fake()),
    //                     "y",
    //                     Position::fake()
    //                 )
    //                 .into()
    //             ),
    //             "x.y"
    //         );
    //     }

    //     mod list {
    //         use super::*;

    //         #[test]
    //         fn format_empty() {
    //             assert_eq!(
    //                 format(
    //                     &List::new(types::None::new(Position::fake()), vec![], Position::fake())
    //                         .into()
    //                 ),
    //                 "[none]"
    //             );
    //         }

    //         #[test]
    //         fn format_element() {
    //             assert_eq!(
    //                 format(
    //                     &List::new(
    //                         types::None::new(Position::fake()),
    //                         vec![ListElement::Single(None::new(Position::fake()).into())],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "[none none]"
    //             );
    //         }

    //         #[test]
    //         fn format_two_elements() {
    //             assert_eq!(
    //                 format(
    //                     &List::new(
    //                         types::None::new(Position::fake()),
    //                         vec![
    //                             ListElement::Single(None::new(Position::fake()).into()),
    //                             ListElement::Single(None::new(Position::fake()).into())
    //                         ],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "[none none, none]"
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line() {
    //             assert_eq!(
    //                 format(
    //                     &List::new(
    //                         types::None::new(Position::fake()),
    //                         vec![ListElement::Single(None::new(line_position(2)).into())],
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     [none
    //                       none,
    //                     ]
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line_with_two_elements() {
    //             assert_eq!(
    //                 format(
    //                     &List::new(
    //                         types::Number::new(Position::fake()),
    //                         vec![
    //                             ListElement::Single(
    //                                 Number::new(
    //                                     NumberRepresentation::FloatingPoint("1".into()),
    //                                     line_position(2)
    //                                 )
    //                                 .into()
    //                             ),
    //                             ListElement::Single(
    //                                 Number::new(
    //                                     NumberRepresentation::FloatingPoint("2".into()),
    //                                     Position::fake()
    //                                 )
    //                                 .into()
    //                             )
    //                         ],
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     [number
    //                       1,
    //                       2,
    //                     ]
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }

    //         #[test]
    //         fn format_comprehension() {
    //             assert_eq!(
    //                 format(
    //                     &ListComprehension::new(
    //                         types::None::new(Position::fake()),
    //                         None::new(Position::fake()),
    //                         "x",
    //                         Variable::new("xs", Position::fake()),
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "[none none for x in xs]"
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line_comprehension() {
    //             assert_eq!(
    //                 format(
    //                     &ListComprehension::new(
    //                         types::None::new(Position::fake()),
    //                         None::new(line_position(2)),
    //                         "x",
    //                         Variable::new("xs", Position::fake()),
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     [none
    //                       none
    //                       for x in xs
    //                     ]
    //                     "
    //                 )
    //                 .trim()
    //             );
    //         }
    //     }

    //     mod map {
    //         use super::*;

    //         #[test]
    //         fn format_empty() {
    //             assert_eq!(
    //                 format(
    //                     &Map::new(
    //                         types::ByteString::new(Position::fake()),
    //                         types::Number::new(Position::fake()),
    //                         vec![],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "{string: number}"
    //             );
    //         }

    //         #[test]
    //         fn format_entry() {
    //             assert_eq!(
    //                 format(
    //                     &Map::new(
    //                         types::ByteString::new(Position::fake()),
    //                         types::Number::new(Position::fake()),
    //                         vec![MapEntry::new(
    //                             ByteString::new("foo", Position::fake()),
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("42".into()),
    //                                 Position::fake()
    //                             ),
    //                             Position::fake()
    //                         )
    //                         .into()],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "{string: number \"foo\": 42}"
    //             );
    //         }

    //         #[test]
    //         fn format_two_entries() {
    //             assert_eq!(
    //                 format(
    //                     &Map::new(
    //                         types::ByteString::new(Position::fake()),
    //                         types::Number::new(Position::fake()),
    //                         vec![
    //                             MapEntry::new(
    //                                 ByteString::new("foo", Position::fake()),
    //                                 Number::new(
    //                                     NumberRepresentation::FloatingPoint("1".into()),
    //                                     Position::fake()
    //                                 ),
    //                                 Position::fake()
    //                             )
    //                             .into(),
    //                             MapEntry::new(
    //                                 ByteString::new("bar", Position::fake()),
    //                                 Number::new(
    //                                     NumberRepresentation::FloatingPoint("2".into()),
    //                                     Position::fake()
    //                                 ),
    //                                 Position::fake()
    //                             )
    //                             .into()
    //                         ],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "{string: number \"foo\": 1, \"bar\": 2}"
    //             );
    //         }

    //         #[test]
    //         fn format_removal() {
    //             assert_eq!(
    //                 format(
    //                     &Map::new(
    //                         types::ByteString::new(Position::fake()),
    //                         types::Number::new(Position::fake()),
    //                         vec![MapElement::Removal(
    //                             ByteString::new("foo", Position::fake()).into()
    //                         )],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "{string: number \"foo\"}"
    //             );
    //         }

    //         #[test]
    //         fn format_map() {
    //             assert_eq!(
    //                 format(
    //                     &Map::new(
    //                         types::ByteString::new(Position::fake()),
    //                         types::Number::new(Position::fake()),
    //                         vec![MapElement::Map(
    //                             Variable::new("xs", Position::fake()).into()
    //                         )],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "{string: number ...xs}"
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line() {
    //             assert_eq!(
    //                 format(
    //                     &Map::new(
    //                         types::ByteString::new(Position::fake()),
    //                         types::Number::new(Position::fake()),
    //                         vec![MapEntry::new(
    //                             ByteString::new("foo", Position::fake()),
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("1".into()),
    //                                 Position::fake()
    //                             ),
    //                             line_position(2)
    //                         )
    //                         .into()],
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     {string: number
    //                       \"foo\": 1,
    //                     }
    //                     "
    //                 )
    //                 .trim(),
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line_with_two_entries() {
    //             assert_eq!(
    //                 format(
    //                     &Map::new(
    //                         types::ByteString::new(Position::fake()),
    //                         types::Number::new(Position::fake()),
    //                         vec![
    //                             MapEntry::new(
    //                                 ByteString::new("foo", Position::fake()),
    //                                 Number::new(
    //                                     NumberRepresentation::FloatingPoint("1".into()),
    //                                     Position::fake()
    //                                 ),
    //                                 line_position(2)
    //                             )
    //                             .into(),
    //                             MapEntry::new(
    //                                 ByteString::new("bar", Position::fake()),
    //                                 Number::new(
    //                                     NumberRepresentation::FloatingPoint("2".into()),
    //                                     Position::fake()
    //                                 ),
    //                                 Position::fake()
    //                             )
    //                             .into()
    //                         ],
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     {string: number
    //                       \"foo\": 1,
    //                       \"bar\": 2,
    //                     }
    //                     "
    //                 )
    //                 .trim(),
    //             );
    //         }
    //     }

    //     mod record {
    //         use super::*;

    //         #[test]
    //         fn format_empty() {
    //             assert_eq!(
    //                 format(&Record::new("foo", None, vec![], Position::fake()).into()),
    //                 "foo{}"
    //             );
    //         }

    //         #[test]
    //         fn format_field() {
    //             assert_eq!(
    //                 format(
    //                     &Record::new(
    //                         "foo",
    //                         None,
    //                         vec![RecordField::new(
    //                             "x",
    //                             None::new(Position::fake()),
    //                             Position::fake()
    //                         )],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "foo{x: none}"
    //             );
    //         }

    //         #[test]
    //         fn format_two_fields() {
    //             assert_eq!(
    //                 format(
    //                     &Record::new(
    //                         "foo",
    //                         None,
    //                         vec![
    //                             RecordField::new(
    //                                 "x",
    //                                 Number::new(
    //                                     NumberRepresentation::FloatingPoint("1".into()),
    //                                     Position::fake()
    //                                 ),
    //                                 Position::fake()
    //                             ),
    //                             RecordField::new(
    //                                 "y",
    //                                 Number::new(
    //                                     NumberRepresentation::FloatingPoint("2".into()),
    //                                     Position::fake()
    //                                 ),
    //                                 Position::fake()
    //                             )
    //                         ],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "foo{x: 1, y: 2}"
    //             );
    //         }

    //         #[test]
    //         fn format_update() {
    //             assert_eq!(
    //                 format(
    //                     &Record::new(
    //                         "foo",
    //                         Some(Variable::new("r", Position::fake()).into()),
    //                         vec![RecordField::new(
    //                             "x",
    //                             None::new(Position::fake()),
    //                             Position::fake()
    //                         )],
    //                         Position::fake()
    //                     )
    //                     .into()
    //                 ),
    //                 "foo{...r, x: none}"
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line() {
    //             assert_eq!(
    //                 format(
    //                     &Record::new(
    //                         "foo",
    //                         None,
    //                         vec![RecordField::new(
    //                             "x",
    //                             None::new(Position::fake()),
    //                             line_position(2)
    //                         )],
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     foo{
    //                       x: none,
    //                     }
    //                     "
    //                 )
    //                 .trim(),
    //             );
    //         }

    //         #[test]
    //         fn format_multi_line_with_two_fields() {
    //             assert_eq!(
    //                 format(
    //                     &Record::new(
    //                         "foo",
    //                         None,
    //                         vec![
    //                             RecordField::new(
    //                                 "x",
    //                                 None::new(Position::fake()),
    //                                 line_position(2)
    //                             ),
    //                             RecordField::new(
    //                                 "y",
    //                                 None::new(Position::fake()),
    //                                 line_position(2)
    //                             )
    //                         ],
    //                         line_position(1)
    //                     )
    //                     .into()
    //                 ),
    //                 indoc!(
    //                     "
    //                     foo{
    //                       x: none,
    //                       y: none,
    //                     }
    //                     "
    //                 )
    //                 .trim(),
    //             );
    //         }
    //     }
    // }

    // mod comment {
    //     use super::*;

    //     #[test]
    //     fn format_comment() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(vec![], vec![], vec![], vec![], Position::fake()),
    //                 &[Comment::new("foo", Position::fake())]
    //             ),
    //             "#foo\n"
    //         );
    //     }

    //     #[test]
    //     fn keep_spaces_between_comments() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(vec![], vec![], vec![], vec![], Position::fake()),
    //                 &[
    //                     Comment::new("foo", line_position(1)),
    //                     Comment::new("bar", line_position(3)),
    //                 ]
    //             ),
    //             indoc!(
    //                 "
    //                 #foo

    //                 #bar
    //                 ",
    //             ),
    //         );
    //     }

    //     #[test]
    //     fn keep_spaces_between_comment_and_next_line() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![Import::new(
    //                         InternalModulePath::new(vec!["Foo".into()]),
    //                         None,
    //                         vec![],
    //                         line_position(3),
    //                     )],
    //                     vec![],
    //                     vec![],
    //                     vec![],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(1))]
    //             ),
    //             indoc!(
    //                 "
    //                 #foo

    //                 import 'Foo
    //                 ",
    //             ),
    //         );
    //     }

    //     #[test]
    //     fn format_import() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![Import::new(
    //                         InternalModulePath::new(vec!["Foo".into()]),
    //                         None,
    //                         vec![],
    //                         line_position(2),
    //                     )],
    //                     vec![],
    //                     vec![],
    //                     vec![],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(1))]
    //             ),
    //             "#foo\nimport 'Foo\n"
    //         );
    //     }

    //     #[test]
    //     fn format_foreign_import() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![ForeignImport::new(
    //                         "foo",
    //                         CallingConvention::Native,
    //                         types::Function::new(
    //                             vec![],
    //                             types::None::new(Position::fake()),
    //                             Position::fake()
    //                         ),
    //                         line_position(2),
    //                     )],
    //                     vec![],
    //                     vec![],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(1))]
    //             ),
    //             "#foo\nimport foreign foo \\() none\n"
    //         );
    //     }

    //     #[test]
    //     fn format_type_definition() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![],
    //                     vec![RecordDefinition::new("foo", vec![], line_position(2)).into()],
    //                     vec![],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(1))]
    //             ),
    //             "#foo\ntype foo {}\n"
    //         );
    //     }

    //     #[test]
    //     fn format_type_alias() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![],
    //                     vec![TypeAlias::new(
    //                         "foo",
    //                         types::None::new(Position::fake()),
    //                         line_position(2)
    //                     )
    //                     .into()],
    //                     vec![],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(1))]
    //             ),
    //             "#foo\ntype foo = none\n"
    //         );
    //     }

    //     #[test]
    //     fn format_definition() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![],
    //                     vec![],
    //                     vec![Definition::new(
    //                         "foo",
    //                         Lambda::new(
    //                             vec![],
    //                             types::None::new(Position::fake()),
    //                             Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                             Position::fake(),
    //                         ),
    //                         None,
    //                         line_position(2)
    //                     )],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(1))]
    //             ),
    //             "#foo\nfoo = \\() none { none }\n"
    //         );
    //     }

    //     #[test]
    //     fn format_statement_in_block() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![],
    //                     vec![],
    //                     vec![Definition::new(
    //                         "foo",
    //                         Lambda::new(
    //                             vec![],
    //                             types::None::new(Position::fake()),
    //                             Block::new(
    //                                 vec![Statement::new(
    //                                     Some("x".into()),
    //                                     None::new(Position::fake()),
    //                                     line_position(2)
    //                                 )],
    //                                 None::new(line_position(3)),
    //                                 Position::fake()
    //                             ),
    //                             Position::fake(),
    //                         ),
    //                         None,
    //                         Position::fake()
    //                     )],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(1))]
    //             ),
    //             indoc!(
    //                 "
    //                 foo = \\() none {
    //                   #foo
    //                   x = none
    //                   none
    //                 }
    //                 "
    //             )
    //         );
    //     }

    //     #[test]
    //     fn format_result_expression_in_block() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![],
    //                     vec![],
    //                     vec![Definition::new(
    //                         "foo",
    //                         Lambda::new(
    //                             vec![],
    //                             types::None::new(Position::fake()),
    //                             Block::new(vec![], None::new(line_position(2)), Position::fake()),
    //                             Position::fake(),
    //                         ),
    //                         None,
    //                         Position::fake()
    //                     )],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(1))]
    //             ),
    //             indoc!(
    //                 "
    //                 foo = \\() none {
    //                   #foo
    //                   none
    //                 }
    //                 "
    //             )
    //         );
    //     }

    //     #[test]
    //     fn format_comment_between_statement_and_expression_in_block() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![],
    //                     vec![],
    //                     vec![Definition::new(
    //                         "foo",
    //                         Lambda::new(
    //                             vec![],
    //                             types::None::new(Position::fake()),
    //                             Block::new(
    //                                 vec![Statement::new(
    //                                     Some("x".into()),
    //                                     None::new(Position::fake()),
    //                                     line_position(1)
    //                                 )],
    //                                 None::new(line_position(3)),
    //                                 Position::fake()
    //                             ),
    //                             Position::fake(),
    //                         ),
    //                         None,
    //                         Position::fake()
    //                     )],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(2))]
    //             ),
    //             indoc!(
    //                 "
    //                 foo = \\() none {
    //                   x = none
    //                   #foo
    //                   none
    //                 }
    //                 "
    //             )
    //         );
    //     }

    //     #[test]
    //     fn format_suffix_comment_after_statement() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![],
    //                     vec![],
    //                     vec![Definition::new(
    //                         "foo",
    //                         Lambda::new(
    //                             vec![],
    //                             types::None::new(Position::fake()),
    //                             Block::new(
    //                                 vec![Statement::new(
    //                                     Some("x".into()),
    //                                     None::new(Position::fake()),
    //                                     line_position(2)
    //                                 )],
    //                                 None::new(line_position(3)),
    //                                 Position::fake()
    //                             ),
    //                             Position::fake(),
    //                         ),
    //                         None,
    //                         Position::fake()
    //                     )],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(2))]
    //             ),
    //             indoc!(
    //                 "
    //                 foo = \\() none {
    //                   x = none #foo
    //                   none
    //                 }
    //                 "
    //             )
    //         );
    //     }

    //     #[test]
    //     fn format_space_between_two_statement_comments() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![],
    //                     vec![],
    //                     vec![Definition::new(
    //                         "foo",
    //                         Lambda::new(
    //                             vec![],
    //                             types::None::new(Position::fake()),
    //                             Block::new(
    //                                 vec![
    //                                     Statement::new(
    //                                         Some("x".into()),
    //                                         None::new(Position::fake()),
    //                                         line_position(3)
    //                                     ),
    //                                     Statement::new(
    //                                         Some("y".into()),
    //                                         None::new(Position::fake()),
    //                                         line_position(6)
    //                                     )
    //                                 ],
    //                                 None::new(line_position(7)),
    //                                 Position::fake()
    //                             ),
    //                             Position::fake(),
    //                         ),
    //                         None,
    //                         Position::fake()
    //                     )],
    //                     Position::fake()
    //                 ),
    //                 &[
    //                     Comment::new("foo", line_position(2)),
    //                     Comment::new("bar", line_position(5))
    //                 ]
    //             ),
    //             indoc!(
    //                 "
    //                 foo = \\() none {
    //                   #foo
    //                   x = none

    //                   #bar
    //                   y = none
    //                   none
    //                 }
    //                 "
    //             )
    //         );
    //     }

    //     #[test]
    //     fn format_suffix_comment_on_function_argument() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![],
    //                     vec![],
    //                     vec![Definition::new(
    //                         "foo",
    //                         Lambda::new(
    //                             vec![Argument::new("x", types::None::new(line_position(2)))],
    //                             types::None::new(Position::fake()),
    //                             Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                             Position::fake(),
    //                         ),
    //                         None,
    //                         line_position(1)
    //                     )],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(2))]
    //             ),
    //             indoc!(
    //                 "
    //                 foo = \\(
    //                   x none, #foo
    //                 ) none {
    //                   none
    //                 }
    //                 "
    //             )
    //         );
    //     }

    //     #[test]
    //     fn format_block_comment_on_function_argument() {
    //         assert_eq!(
    //             format(
    //                 &Module::new(
    //                     vec![],
    //                     vec![],
    //                     vec![],
    //                     vec![Definition::new(
    //                         "foo",
    //                         Lambda::new(
    //                             vec![Argument::new("x", types::None::new(line_position(3)))],
    //                             types::None::new(Position::fake()),
    //                             Block::new(vec![], None::new(Position::fake()), Position::fake()),
    //                             Position::fake(),
    //                         ),
    //                         None,
    //                         line_position(1)
    //                     )],
    //                     Position::fake()
    //                 ),
    //                 &[Comment::new("foo", line_position(2))]
    //             ),
    //             indoc!(
    //                 "
    //                 foo = \\(
    //                   #foo
    //                   x none,
    //                 ) none {
    //                   none
    //                 }
    //                 "
    //             )
    //         );
    //     }
    // }
}
