use crate::hir::*;

pub fn visit(module: &Module, mut visit: impl FnMut(&Expression)) {
    for definition in module.definitions() {
        visit_definition(definition, &mut visit);
    }
}

fn visit_definition(definition: &Definition, visit: &mut impl FnMut(&Expression)) {
    visit_expression(definition.lambda().body(), visit)
}

fn visit_expression(expression: &Expression, visit: &mut impl FnMut(&Expression)) {
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
            visit_expression(if_.argument());
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
        Expression::Lambda(lambda) => visit_expression(lambda.body()),
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
        Expression::Operation(operation) => match operation {
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
            Operation::Try(operation) => {
                visit_expression(operation.expression());
            }
        },
        Expression::RecordConstruction(construction) => {
            for element in construction.elements() {
                visit_expression(element.expression());
            }
        }
        Expression::RecordDeconstruction(deconstruction) => {
            visit_expression(deconstruction.record());
        }
        Expression::RecordUpdate(update) => {
            visit_expression(update.record());

            for element in update.elements() {
                visit_expression(element.expression());
            }
        }
        Expression::Boolean(_)
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Variable(_) => Default::default(),
    }
}
