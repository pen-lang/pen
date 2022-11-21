use crate::{
    ir::*,
    types::{self, Type},
};

pub fn visit<'a>(module: &'a Module, mut visit: impl FnMut(&'a Type)) {
    for definition in module.type_definitions() {
        visit_type_definition(definition, &mut visit);
    }

    for alias in module.type_aliases() {
        visit_type_alias(alias, &mut visit);
    }

    for declaration in module.foreign_declarations() {
        visit_foreign_declaration(declaration, &mut visit);
    }

    for declaration in module.function_declarations() {
        visit_function_declaration(declaration, &mut visit);
    }

    for definition in module.function_definitions() {
        visit_function_definition(definition, &mut visit);
    }
}

fn visit_type_definition<'a>(definition: &'a TypeDefinition, visit: &mut impl FnMut(&'a Type)) {
    for field in definition.fields() {
        visit_type(field.type_(), visit);
    }
}

fn visit_type_alias<'a>(alias: &'a TypeAlias, visit: &mut impl FnMut(&'a Type)) {
    visit_type(alias.type_(), visit)
}

fn visit_foreign_declaration<'a>(
    declaration: &'a ForeignDeclaration,
    visit: &mut impl FnMut(&'a Type),
) {
    visit_type(declaration.type_(), visit)
}

fn visit_function_declaration<'a>(
    declaration: &'a FunctionDeclaration,
    visit: &mut impl FnMut(&'a Type),
) {
    visit_function_type(declaration.type_(), visit);
}

fn visit_function_definition<'a>(
    definition: &'a FunctionDefinition,
    visit: &mut impl FnMut(&'a Type),
) {
    visit_lambda(definition.lambda(), visit)
}

fn visit_lambda<'a>(lambda: &'a Lambda, visit: &mut impl FnMut(&'a Type)) {
    for argument in lambda.arguments() {
        visit_type(argument.type_(), visit);
    }

    visit_type(lambda.result_type(), visit);

    visit_expression(lambda.body(), visit)
}

fn visit_expression<'a>(expression: &'a Expression, visit: &mut impl FnMut(&'a Type)) {
    match expression {
        Expression::Call(call) => {
            if let Some(type_) = call.function_type() {
                visit_type(type_, visit);
            }

            visit_expression(call.function(), visit);

            for argument in call.arguments() {
                visit_expression(argument, visit);
            }
        }
        Expression::If(if_) => {
            visit_expression(if_.condition(), visit);
            visit_expression(if_.then(), visit);
            visit_expression(if_.else_(), visit);
        }
        Expression::IfList(if_) => {
            if let Some(type_) = if_.type_() {
                visit_type(type_, visit);
            }

            visit_expression(if_.list(), visit);
            visit_expression(if_.then(), visit);
            visit_expression(if_.else_(), visit);
        }
        Expression::IfMap(if_) => {
            if let Some(type_) = if_.key_type() {
                visit_type(type_, visit);
            }

            if let Some(type_) = if_.value_type() {
                visit_type(type_, visit);
            }

            visit_expression(if_.map(), visit);
            visit_expression(if_.key(), visit);
            visit_expression(if_.then(), visit);
            visit_expression(if_.else_(), visit);
        }
        Expression::IfType(if_) => {
            visit_expression(if_.argument(), visit);

            for branch in if_.branches() {
                visit_type(branch.type_(), visit);
                visit_expression(branch.expression(), visit);
            }

            if let Some(branch) = if_.else_() {
                if let Some(type_) = branch.type_() {
                    visit_type(type_, visit);
                }

                visit_expression(branch.expression(), visit);
            }
        }
        Expression::TypeCoercion(coercion) => {
            visit_type(coercion.from(), visit);
            visit_type(coercion.to(), visit);

            visit_expression(coercion.argument(), visit);
        }
        Expression::Lambda(lambda) => visit_lambda(lambda, visit),
        Expression::Let(let_) => {
            if let Some(type_) = let_.type_() {
                visit_type(type_, visit);
            }

            visit_expression(let_.bound_expression(), visit);
            visit_expression(let_.expression(), visit);
        }
        Expression::List(list) => {
            visit_type(list.type_(), visit);

            for element in list.elements() {
                visit_expression(
                    match element {
                        ListElement::Multiple(expression) => expression,
                        ListElement::Single(expression) => expression,
                    },
                    visit,
                );
            }
        }
        Expression::ListComprehension(comprehension) => {
            visit_type(comprehension.type_(), visit);

            for branch in comprehension.branches() {
                if let Some(type_) = branch.type_() {
                    visit_type(type_, visit);
                }

                visit_expression(branch.iteratee(), visit);
            }

            visit_expression(comprehension.element(), visit);
        }
        Expression::Map(map) => {
            visit_type(map.key_type(), visit);
            visit_type(map.value_type(), visit);

            for element in map.elements() {
                match element {
                    MapElement::Insertion(entry) => {
                        visit_expression(entry.key(), visit);
                        visit_expression(entry.value(), visit);
                    }
                    MapElement::Map(expression) => visit_expression(expression, visit),
                }
            }
        }
        Expression::Operation(operation) => match operation {
            Operation::Addition(operation) => {
                if let Some(type_) = operation.type_() {
                    visit_type(type_, visit);
                }

                visit_expression(operation.lhs(), visit);
                visit_expression(operation.rhs(), visit);
            }
            Operation::Arithmetic(operation) => {
                visit_expression(operation.lhs(), visit);
                visit_expression(operation.rhs(), visit);
            }
            Operation::Boolean(operation) => {
                visit_expression(operation.lhs(), visit);
                visit_expression(operation.rhs(), visit);
            }
            Operation::Equality(operation) => {
                if let Some(type_) = operation.type_() {
                    visit_type(type_, visit);
                }

                visit_expression(operation.lhs(), visit);
                visit_expression(operation.rhs(), visit);
            }
            Operation::Not(operation) => {
                visit_expression(operation.expression(), visit);
            }
            Operation::Order(operation) => {
                visit_expression(operation.lhs(), visit);
                visit_expression(operation.rhs(), visit);
            }
            Operation::Try(operation) => visit_expression(operation.expression(), visit),
        },
        Expression::RecordConstruction(construction) => {
            visit_type(construction.type_(), visit);

            for field in construction.fields() {
                visit_expression(field.expression(), visit);
            }
        }
        Expression::RecordDeconstruction(deconstruction) => {
            if let Some(type_) = deconstruction.type_() {
                visit_type(type_, visit);
            }

            visit_expression(deconstruction.record(), visit)
        }
        Expression::RecordUpdate(update) => {
            visit_type(update.type_(), visit);

            visit_expression(update.record(), visit);

            for field in update.fields() {
                visit_expression(field.expression(), visit);
            }
        }
        Expression::Thunk(thunk) => {
            if let Some(type_) = thunk.type_() {
                visit_type(type_, visit);
            }

            visit_expression(thunk.expression(), visit);
        }
        Expression::Boolean(_)
        | Expression::BuiltInFunction(_)
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Variable(_) => {}
    }
}

fn visit_type<'a>(type_: &'a Type, visit: &mut impl FnMut(&'a Type)) {
    visit(type_);

    let mut visit_type = |type_| visit_type(type_, visit);

    match type_ {
        Type::Function(function) => visit_function_type(function, visit),
        Type::List(list) => visit_type(list.element()),
        Type::Map(map) => {
            visit_type(map.key());
            visit_type(map.value())
        }
        Type::Union(union) => {
            visit_type(union.lhs());
            visit_type(union.rhs())
        }
        Type::Any(_)
        | Type::Boolean(_)
        | Type::Error(_)
        | Type::None(_)
        | Type::Number(_)
        | Type::Record(_)
        | Type::Reference(_)
        | Type::String(_) => {}
    }
}

fn visit_function_type<'a>(function: &'a types::Function, visit: &mut impl FnMut(&'a Type)) {
    visit_type(function.result(), visit);

    for argument in function.arguments() {
        visit_type(argument, visit);
    }
}
