use crate::ir::*;

pub fn visit<'a>(module: &'a Module, mut visit: impl FnMut(&'a Expression)) {
    for definition in module.function_definitions() {
        visit_definition(definition, &mut visit);
    }
}

fn visit_definition<'a>(
    definition: &'a FunctionDefinition,
    visit: &mut impl FnMut(&'a Expression),
) {
    visit_lambda(definition.lambda(), visit)
}

fn visit_lambda<'a>(lambda: &'a Lambda, visit: &mut impl FnMut(&'a Expression)) {
    visit_expression(lambda.body(), visit)
}

fn visit_expression<'a>(expression: &'a Expression, visit: &mut impl FnMut(&'a Expression)) {
    visit(expression);

    let mut visit_expression = |expression| visit_expression(expression, visit);

    match expression {
        Expression::Call(call) => {
            visit_expression(call.function());

            for argument in call.arguments() {
                visit_expression(argument);
            }
        }
        Expression::If(if_) => {
            visit_expression(if_.condition());
            visit_expression(if_.then());
            visit_expression(if_.else_());
        }
        Expression::IfList(if_) => {
            visit_expression(if_.list());
            visit_expression(if_.then());
            visit_expression(if_.else_());
        }
        Expression::IfMap(if_) => {
            visit_expression(if_.map());
            visit_expression(if_.key());
            visit_expression(if_.then());
            visit_expression(if_.else_());
        }
        Expression::IfType(if_) => {
            visit_expression(if_.argument());

            for branch in if_.branches() {
                visit_expression(branch.expression());
            }

            if let Some(branch) = if_.else_() {
                visit_expression(branch.expression());
            }
        }
        Expression::TypeCoercion(coercion) => {
            visit_expression(coercion.argument());
        }
        Expression::Lambda(lambda) => visit_lambda(lambda, visit),
        Expression::Let(let_) => {
            visit_expression(let_.bound_expression());
            visit_expression(let_.expression());
        }
        Expression::List(list) => {
            for element in list.elements() {
                visit_expression(match element {
                    ListElement::Multiple(expression) => expression,
                    ListElement::Single(expression) => expression,
                });
            }
        }
        Expression::ListComprehension(comprehension) => {
            for branch in comprehension.branches() {
                visit_expression(branch.iteratee());
            }

            visit_expression(comprehension.element());
        }
        Expression::Map(map) => {
            for element in map.elements() {
                match element {
                    MapElement::Insertion(entry) => {
                        visit_expression(entry.key());
                        visit_expression(entry.value());
                    }
                    MapElement::Map(expression) => visit_expression(expression),
                }
            }
        }
        Expression::Operation(operation) => match operation {
            Operation::Addition(operation) => {
                visit_expression(operation.lhs());
                visit_expression(operation.rhs());
            }
            Operation::Arithmetic(operation) => {
                visit_expression(operation.lhs());
                visit_expression(operation.rhs());
            }
            Operation::Boolean(operation) => {
                visit_expression(operation.lhs());
                visit_expression(operation.rhs());
            }
            Operation::Equality(operation) => {
                visit_expression(operation.lhs());
                visit_expression(operation.rhs());
            }
            Operation::Not(operation) => {
                visit_expression(operation.expression());
            }
            Operation::Order(operation) => {
                visit_expression(operation.lhs());
                visit_expression(operation.rhs());
            }
            Operation::Try(operation) => visit_expression(operation.expression()),
        },
        Expression::RecordConstruction(construction) => {
            for field in construction.fields() {
                visit_expression(field.expression());
            }
        }
        Expression::RecordDeconstruction(deconstruction) => {
            visit_expression(deconstruction.record())
        }
        Expression::RecordUpdate(update) => {
            visit_expression(update.record());

            for field in update.fields() {
                visit_expression(field.expression());
            }
        }
        Expression::Thunk(thunk) => visit_expression(thunk.expression()),
        Expression::Boolean(_)
        | Expression::BuiltInFunction(_)
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Variable(_) => {}
    }
}
