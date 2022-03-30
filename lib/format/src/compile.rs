use crate::comment;
use crate::ir;
use ast::{types::Type, *};
use position::Position;
use std::str;

// TODO Consider introducing a minimum editor width to enforce single-line
// formats in some occasions.

// pub fn format(module: &Module, comments: &[Comment]) -> ir::Document {
//     let comments = comment::sort(comments);

//     let (external_imports, internal_imports) = module
//         .imports()
//         .iter()
//         .partition::<Vec<_>, _>(|import| matches!(import.module_path(), ModulePath::External(_)));

//     let (external_imports, comments) = compile_imports(&external_imports, &comments);
//     let (internal_imports, comments) = compile_imports(&internal_imports, comments);
//     let (foreign_imports, mut comments) =
//         compile_foreign_imports(module.foreign_imports(), comments);

//     let mut sections = vec![external_imports, internal_imports, foreign_imports];

//     for definition in module.type_definitions() {
//         let (definition, new_comments) = compile_type_definition(definition, comments);

//         sections.push(definition);
//         comments = new_comments;
//     }

//     for definition in module.definitions() {
//         let (definition, new_comments) = compile_definition(definition, comments);

//         sections.push(definition);
//         comments = new_comments;
//     }

//     compile_spaces(
//         sections
//             .into_iter()
//             .filter(|string| !string.is_empty())
//             .collect::<Vec<_>>()
//             .join("\n\n")
//             + "\n\n"
//             + &compile_all_comments(comments, None),
//     )
// }

// fn compile_spaces(string: impl AsRef<str>) -> ir::Document {
//     let string = regex::Regex::new(r"[ \t]*\n")
//         .unwrap()
//         .replace_all(string.as_ref(), "\n");

//     let string = regex::Regex::new(r"\n\n\n+")
//         .unwrap()
//         .replace_all(&string, "\n\n");

//     string.trim().to_owned() + "\n"
// }

// fn compile_imports<'c>(
//     imports: &[&Import],
//     mut comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let mut strings = vec![];

//     for import in imports {
//         let (import, new_comments) = compile_import(import, comments);

//         strings.push(import);
//         comments = new_comments;
//     }

//     strings.sort();

//     (strings.join("\n"), comments)
// }

// fn compile_import<'c>(import: &Import, comments: &'c [Comment]) -> (ir::Document, &'c [Comment]) {
//     let (block_comment, comments) = compile_block_comment(comments, import.position());

//     (
//         block_comment
//             + &["import".into(), compile_module_path(import.module_path())]
//                 .into_iter()
//                 .chain(import.prefix().map(|prefix| format!("as {}", prefix)))
//                 .chain(if import.unqualified_names().is_empty() {
//                     None
//                 } else {
//                     Some(format!("{{ {} }}", import.unqualified_names().join(", ")))
//                 })
//                 .collect::<Vec<_>>()
//                 .join(" "),
//         comments,
//     )
// }

// fn compile_foreign_imports<'c>(
//     imports: &[ForeignImport],
//     mut comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let mut strings = vec![];

//     for import in imports {
//         let (string, new_comments) = compile_foreign_import(import, comments);

//         strings.push(string);
//         comments = new_comments;
//     }

//     (strings.join("\n"), comments)
// }

// fn compile_module_path(path: &ModulePath) -> ir::Document {
//     match path {
//         ModulePath::External(path) => {
//             format!(
//                 "{}'{}",
//                 path.package(),
//                 compile_module_path_components(path.components())
//             )
//         }
//         ModulePath::Internal(path) => {
//             format!("'{}", compile_module_path_components(path.components()))
//         }
//     }
// }

// fn compile_module_path_components(components: &[String]) -> ir::Document {
//     components.join("'")
// }

// fn compile_foreign_import<'c>(
//     import: &ForeignImport,
//     comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let (block_comment, comments) = compile_block_comment(comments, import.position());

//     (
//         block_comment
//             + &["import foreign".into()]
//                 .into_iter()
//                 .chain(match import.calling_convention() {
//                     CallingConvention::C => Some("\"c\"".into()),
//                     CallingConvention::Native => None,
//                 })
//                 .chain([import.name().into(), compile_type(import.type_())])
//                 .collect::<Vec<_>>()
//                 .join(" "),
//         comments,
//     )
// }

// fn compile_type_definition<'c>(
//     definition: &TypeDefinition,
//     comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     match definition {
//         TypeDefinition::RecordDefinition(definition) => {
//             compile_record_definition(definition, comments)
//         }
//         TypeDefinition::TypeAlias(alias) => compile_type_alias(alias, comments),
//     }
// }

// fn compile_record_definition<'c>(
//     definition: &RecordDefinition,
//     comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let (block_comment, comments) = compile_block_comment(comments, definition.position());
//     let head = ir::Join::new(
//         vec!["type".into(), definition.name().into(), "{".into()],
//         " ",
//     );

//     (
//         vec![
//             block_comment,
//             ir::Group::new(ir::Join::new(
//                 [ir::Document::from(head)]
//                     .into_iter()
//                     .chain(definition.fields().iter().map(|field| {
//                         ir::Indent::new(vec![
//                             ir::Document::from(field.name()),
//                             ":".into(),
//                             " ".into(),
//                             compile_type(field.type_()),
//                         ])
//                         .into()
//                     }))
//                     .chain(["}".into()])
//                     .collect::<Vec<_>>(),
//                 vec![",".into(), ir::Document::SoftBreak],
//             ))
//             .into(),
//         ]
//         .into(),
//         comments,
//     )
// }

// fn compile_type_alias<'c>(
//     alias: &TypeAlias,
//     comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let (block_comment, comments) = compile_block_comment(comments, alias.position());

//     (
//         block_comment
//             + &[
//                 "type".into(),
//                 alias.name().into(),
//                 "=".into(),
//                 compile_type(alias.type_()),
//             ]
//             .join(" "),
//         comments,
//     )
// }

// fn compile_definition<'c>(
//     definition: &Definition,
//     comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let (block_comment, comments) = compile_block_comment(comments, definition.position());
//     let (lambda, comments) = compile_lambda(definition.lambda(), comments);

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

// fn compile_type(type_: &Type) -> ir::Document {
//     match type_ {
//         Type::Any(_) => "any".into(),
//         Type::Boolean(_) => "boolean".into(),
//         Type::Function(function) => vec![
//             ir::Join::new(
//                 ["\\(".into()]
//                     .into_iter()
//                     .chain(function.arguments().iter().map(compile_type))
//                     .chain([")".into()])
//                     .collect::<Vec<_>>()
//                     .into(),
//                 ", ",
//             )
//             .into(),
//             compile_type(function.result()),
//         ]
//         .into(),
//         Type::List(list) => vec!["[".into(), compile_type(list.element()), "]".into()].into(),
//         Type::Map(map) => vec![
//             "{".into(),
//             compile_type(map.key()),
//             ": ".into(),
//             compile_type(map.value()),
//             "}".into(),
//         ]
//         .into(),
//         Type::None(_) => "none".into(),
//         Type::Number(_) => "number".into(),
//         Type::Record(record) => record.name().into(),
//         Type::Reference(reference) => reference.name().into(),
//         Type::String(_) => "string".into(),
//         Type::Union(union) => {
//             let type_ = compile_type(union.lhs());

//             vec![
//                 if union.lhs().is_function() {
//                     vec!["(".into(), type_, ")".into()].into()
//                 } else {
//                     type_
//                 },
//                 " | ".into(),
//                 compile_type(union.rhs()),
//             ]
//             .into()
//         }
//     }
// }

// fn compile_lambda<'c>(
//     lambda: &Lambda,
//     mut comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let arguments = lambda
//         .arguments()
//         .iter()
//         .map(|argument| format!("{} {}", argument.name(), compile_type(argument.type_())))
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
//             let (block_comment, new_comments) = compile_block_comment(comments, position);
//             let (suffix_comment, new_comments) = compile_suffix_comment(new_comments, position);

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
//         compile_block(lambda.body(), comments)
//     } else {
//         compile_multi_line_block(lambda.body(), comments)
//     };

//     (
//         format!(
//             "\\{} {} {}",
//             arguments,
//             compile_type(lambda.result_type()),
//             body
//         ),
//         comments,
//     )
// }

// fn compile_block<'c>(block: &Block, comments: &'c [Comment]) -> (ir::Document, &'c [Comment]) {
//     let (expression, new_comments) = compile_expression(block.expression(), comments);

//     if block.statements().is_empty() && is_single_line(&expression) {
//         (["{", &expression, "}"].join(" "), new_comments)
//     } else {
//         compile_multi_line_block(block, comments)
//     }
// }

// fn compile_multi_line_block<'c>(
//     block: &Block,
//     mut comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let mut statements = vec![];

//     for (statement, next_position) in block.statements().iter().zip(
//         block
//             .statements()
//             .iter()
//             .skip(1)
//             .map(|statement| statement.position())
//             .chain([block.expression().position()]),
//     ) {
//         let (block_comment, new_comments) = compile_block_comment(comments, statement.position());
//         // TODO Use end positions of spans when they are available.
//         let line_count = next_position.line_number() as isize
//             - statement.position().line_number() as isize
//             - comment::split_before(new_comments, next_position.line_number())
//                 .0
//                 .len() as isize;
//         let (statement_string, new_comments) = compile_statement(statement, new_comments);
//         let (suffix_comment, new_comments) =
//             compile_suffix_comment(new_comments, statement.position());

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

//     let (block_comment, comments) = compile_block_comment(comments, block.expression().position());
//     let (expression, comments) = compile_expression(block.expression(), comments);

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

// fn compile_statement<'c>(
//     statement: &Statement,
//     comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let (expression, comments) = compile_expression(statement.expression(), comments);

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

// fn compile_expression<'c>(
//     expression: &Expression,
//     mut comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     match expression {
//         Expression::BinaryOperation(operation) => compile_binary_operation(operation, comments),
//         Expression::Call(call) => {
//             let (function, mut comments) = compile_expression(call.function(), comments);
//             let head = format!("{}(", function);
//             let mut arguments = vec![];

//             for argument in call.arguments() {
//                 let (expression, new_comments) = compile_expression(argument, comments);

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
//         Expression::If(if_) => compile_if(if_, comments),
//         Expression::IfList(if_) => {
//             let (list, comments) = compile_expression(if_.list(), comments);
//             let (then, comments) = compile_multi_line_block(if_.then(), comments);
//             let (else_, comments) = compile_multi_line_block(if_.else_(), comments);

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
//             let (map, comments) = compile_expression(if_.map(), comments);
//             let (key, comments) = compile_expression(if_.key(), comments);
//             let (then, comments) = compile_multi_line_block(if_.then(), comments);
//             let (else_, comments) = compile_multi_line_block(if_.else_(), comments);

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
//         Expression::IfType(if_) => compile_if_type(if_, comments),
//         Expression::Lambda(lambda) => compile_lambda(lambda, comments),
//         Expression::List(list) => {
//             let type_ = compile_type(list.type_());
//             let mut elements = vec![];

//             for element in list.elements() {
//                 let (element, new_comments) = compile_list_element(element, comments);

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
//             let type_ = compile_type(comprehension.type_());
//             let (element, comments) = compile_expression(comprehension.element(), comments);
//             let (list, comments) = compile_expression(comprehension.list(), comments);

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
//             let type_ = compile_type(map.key_type()) + ": " + &compile_type(map.value_type());
//             let mut elements = vec![];

//             for element in map.elements() {
//                 let (element, new_comments) = compile_map_element(element, comments);

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
//                 let (record, comments) = compile_expression(record, comments);

//                 (Some(format!("...{}", record)), comments)
//             } else {
//                 (None, comments)
//             };
//             let mut elements = old_record.into_iter().collect::<Vec<_>>();

//             for field in record.fields() {
//                 let (expression, new_comments) = compile_expression(field.expression(), comments);

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
//             let (record, comments) = compile_expression(deconstruction.expression(), comments);

//             (format!("{}.{}", record, deconstruction.name()), comments)
//         }
//         Expression::SpawnOperation(operation) => {
//             let (lambda, comments) = compile_lambda(operation.function(), comments);

//             (format!("go {}", lambda), comments)
//         }
//         Expression::String(string) => (format!("\"{}\"", string.value()), comments),
//         Expression::UnaryOperation(operation) => {
//             let (operand, comments) = compile_expression(operation.expression(), comments);
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

// fn compile_if<'c>(if_: &If, original_comments: &'c [Comment]) -> (ir::Document, &'c [Comment]) {
//     let (branches, comments) = {
//         let mut branches = vec![];
//         let mut comments = original_comments;

//         for branch in if_.branches() {
//             let (expression, new_comments) = compile_expression(branch.condition(), comments);
//             let (block, new_comments) = compile_block(branch.block(), new_comments);

//             branches.push(expression + " " + &block);
//             comments = new_comments;
//         }

//         (branches, comments)
//     };
//     let (else_, comments) = compile_block(if_.else_(), comments);

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
//             let (expression, new_comments) = compile_expression(branch.condition(), comments);
//             let (block, new_comments) = compile_multi_line_block(branch.block(), new_comments);

//             parts.extend(["if".into(), expression, block, "else".into()]);
//             comments = new_comments;
//         }

//         let (else_, comments) = compile_multi_line_block(if_.else_(), comments);

//         parts.push(else_);

//         (parts.join(" "), comments)
//     }
// }

// fn compile_if_type<'c>(
//     if_: &IfType,
//     original_comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let (argument, original_comments) = compile_expression(if_.argument(), original_comments);
//     let (branches, comments) = {
//         let mut branches = vec![];
//         let mut comments = original_comments;

//         for branch in if_.branches() {
//             let (block, new_comments) = compile_block(branch.block(), comments);

//             branches.push(compile_type(branch.type_()) + " " + &block);
//             comments = new_comments;
//         }

//         (branches, comments)
//     };
//     let (else_, comments) = if let Some(block) = if_.else_() {
//         let (block, comments) = compile_block(block, comments);

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
//             let (block, new_comments) = compile_multi_line_block(branch.block(), comments);

//             branches.push(compile_type(branch.type_()) + " " + &block);
//             comments = new_comments;
//         }

//         let (else_, comments) = if let Some(block) = if_.else_() {
//             let (block, comments) = compile_multi_line_block(block, comments);

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

// fn compile_list_element<'c>(
//     element: &ListElement,
//     comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     match element {
//         ListElement::Multiple(expression) => {
//             let (expression, comments) = compile_expression(expression, comments);

//             (format!("...{}", expression), comments)
//         }
//         ListElement::Single(expression) => compile_expression(expression, comments),
//     }
// }

// fn compile_map_element<'c>(
//     element: &MapElement,
//     comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     match element {
//         MapElement::Map(expression) => {
//             let (expression, comments) = compile_expression(expression, comments);

//             (format!("...{}", expression), comments)
//         }
//         MapElement::Insertion(entry) => {
//             let (key, comments) = compile_expression(entry.key(), comments);
//             let (value, comments) = compile_expression(entry.value(), comments);

//             (format!("{}: {}", key, value), comments)
//         }
//         MapElement::Removal(expression) => compile_expression(expression, comments),
//     }
// }

// fn compile_binary_operation<'c>(
//     operation: &BinaryOperation,
//     comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let single_line =
//         operation.lhs().position().line_number() == operation.rhs().position().line_number();
//     let operator = compile_binary_operator(operation.operator()).into();
//     let (lhs, comments) = compile_operand(operation.lhs(), operation.operator(), comments);
//     let (rhs, comments) = compile_operand(operation.rhs(), operation.operator(), comments);

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

// fn compile_operand<'c>(
//     operand: &Expression,
//     parent_operator: BinaryOperator,
//     comments: &'c [Comment],
// ) -> (ir::Document, &'c [Comment]) {
//     let (string, comments) = compile_expression(operand, comments);

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

// fn compile_binary_operator(operator: BinaryOperator) -> &'static str {
//     match operator {
//         BinaryOperator::Or => "|",
//         BinaryOperator::And => "&",
//         BinaryOperator::Equal => "==",
//         BinaryOperator::NotEqual => "!=",
//         BinaryOperator::LessThan => "<",
//         BinaryOperator::LessThanOrEqual => "<=",
//         BinaryOperator::GreaterThan => ">",
//         BinaryOperator::GreaterThanOrEqual => ">=",
//         BinaryOperator::Add => "+",
//         BinaryOperator::Subtract => "-",
//         BinaryOperator::Multiply => "*",
//         BinaryOperator::Divide => "/",
//     }
// }

// fn operator_priority(operator: BinaryOperator) -> usize {
//     match operator {
//         BinaryOperator::Or => 1,
//         BinaryOperator::And => 2,
//         BinaryOperator::Equal
//         | BinaryOperator::NotEqual
//         | BinaryOperator::LessThan
//         | BinaryOperator::LessThanOrEqual
//         | BinaryOperator::GreaterThan
//         | BinaryOperator::GreaterThanOrEqual => 3,
//         BinaryOperator::Add | BinaryOperator::Subtract => 4,
//         BinaryOperator::Multiply | BinaryOperator::Divide => 5,
//     }
// }

// fn compile_suffix_comment<'c>(
//     comments: &'c [Comment],
//     position: &Position,
// ) -> (Option<ir::Document>, &'c [Comment]) {
//     let (comment, comments) = comment::split_current(comments, position.line_number());

//     (
//         comment.map(|comment| {
//             vec![
//                 ir::Document::Comment("#".to_owned() + comment.line()),
//                 ir::Document::HardBreak,
//             ]
//             .into()
//         }),
//         comments,
//     )
// }

// fn compile_block_comment<'c>(
//     comments: &'c [Comment],
//     position: &Position,
// ) -> (ir::Document, &'c [Comment]) {
//     let (before, after) = comment::split_before(comments, position.line_number());

//     (
//         compile_all_comments(before, Some(position.line_number())),
//         after,
//     )
// }

// fn compile_all_comments(comments: &[Comment], last_line_number: Option<usize>) -> ir::Document {
//     comments
//         .iter()
//         .zip(
//             comments
//                 .iter()
//                 .skip(1)
//                 .map(|comment| comment.position().line_number())
//                 .chain([last_line_number.unwrap_or(usize::MAX)]),
//         )
//         .flat_map(|(comment, next_line_number)| {
//             [
//                 ir::Document::Comment("#".to_owned() + comment.line().trim_end()),
//                 ir::Document::HardBreak,
//             ]
//             .into_iter()
//             .chain(if comment.position().line_number() + 1 < next_line_number {
//                 Some(ir::Document::HardBreak)
//             } else {
//                 None
//             })
//         })
//         .collect::<Vec<_>>()
//         .into()
// }

// fn indent(string: impl AsRef<str>) -> ir::Document {
//     regex::Regex::new("^|\n")
//         .unwrap()
//         .replace_all(
//             string.as_ref(),
//             "${0}".to_owned() + &" ".repeat(INDENT_DEPTH),
//         )
//         .into()
// }

// fn count_lines(string: &str) -> usize {
//     string.trim().matches('\n').count() + 1
// }

// fn is_single_line(string: impl AsRef<str>) -> bool {
//     !string.as_ref().contains('\n')
// }
