use crate::ir::*;

pub fn transform(module: &Module, transform: &dyn Fn(&Variable) -> Expression) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| transform_definition(definition, transform))
            .collect(),
        module.position().clone(),
    )
}

fn transform_definition(
    definition: &Definition,
    transform: &dyn Fn(&Variable) -> Expression,
) -> Definition {
    Definition::new(
        definition.name(),
        definition.original_name(),
        transform_lambda(definition.lambda(), transform),
        definition.foreign_definition_configuration().cloned(),
        definition.is_public(),
        definition.position().clone(),
    )
}

fn transform_lambda(lambda: &Lambda, transform: &dyn Fn(&Variable) -> Expression) -> Lambda {
    Lambda::new(
        lambda.arguments().to_vec(),
        lambda.result_type().clone(),
        transform_expression(lambda.body(), &|variable| {
            if lambda
                .arguments()
                .iter()
                .any(|argument| argument.name() == variable.name())
            {
                variable.clone().into()
            } else {
                transform(variable)
            }
        }),
        lambda.position().clone(),
    )
}

fn transform_expression(
    expression: &Expression,
    transform: &dyn Fn(&Variable) -> Expression,
) -> Expression {
    match expression {
        Expression::Call(call) => Call::new(
            call.function_type().cloned(),
            transform_expression(call.function(), transform),
            call.arguments()
                .iter()
                .map(|argument| transform_expression(argument, transform))
                .collect(),
            call.position().clone(),
        )
        .into(),
        Expression::If(if_) => If::new(
            transform_expression(if_.condition(), transform),
            transform_expression(if_.then(), transform),
            transform_expression(if_.else_(), transform),
            if_.position().clone(),
        )
        .into(),
        Expression::IfList(if_) => IfList::new(
            if_.type_().cloned(),
            transform_expression(if_.argument(), transform),
            if_.first_name(),
            if_.rest_name(),
            transform_expression(if_.then(), &|variable| {
                if if_.first_name() == variable.name() || if_.rest_name() == variable.name() {
                    variable.clone().into()
                } else {
                    transform(variable)
                }
            }),
            transform_expression(if_.else_(), transform),
            if_.position().clone(),
        )
        .into(),
        Expression::IfType(if_) => {
            let branch_transform = &|variable: &Variable| {
                if if_.name() == variable.name() {
                    variable.clone().into()
                } else {
                    transform(variable)
                }
            };

            IfType::new(
                if_.name(),
                transform_expression(if_.argument(), transform),
                if_.branches()
                    .iter()
                    .map(|branch| {
                        IfTypeBranch::new(
                            branch.type_().clone(),
                            transform_expression(branch.expression(), &branch_transform),
                        )
                    })
                    .collect(),
                if_.else_().map(|branch| {
                    ElseBranch::new(
                        branch.type_().cloned(),
                        transform_expression(branch.expression(), &branch_transform),
                        branch.position().clone(),
                    )
                }),
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => transform_lambda(lambda, transform).into(),
        Expression::Let(let_) => Let::new(
            let_.name().map(String::from),
            let_.type_().cloned(),
            transform_expression(let_.bound_expression(), transform),
            transform_expression(let_.expression(), &|variable| {
                if let_.name() == Some(variable.name()) {
                    variable.clone().into()
                } else {
                    transform(variable)
                }
            }),
            let_.position().clone(),
        )
        .into(),
        Expression::List(list) => List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| match element {
                    ListElement::Multiple(element) => {
                        ListElement::Multiple(transform_expression(element, transform))
                    }
                    ListElement::Single(element) => {
                        ListElement::Single(transform_expression(element, transform))
                    }
                })
                .collect(),
            list.position().clone(),
        )
        .into(),
        Expression::Operation(operation) => transform_operation(operation, transform).into(),
        Expression::RecordConstruction(construction) => RecordConstruction::new(
            construction.type_().clone(),
            construction
                .elements()
                .iter()
                .map(|element| {
                    RecordField::new(
                        element.name(),
                        transform_expression(element.expression(), transform),
                        element.position().clone(),
                    )
                })
                .collect(),
            construction.position().clone(),
        )
        .into(),
        Expression::RecordDeconstruction(deconstruction) => RecordDeconstruction::new(
            deconstruction.type_().cloned(),
            transform_expression(deconstruction.record(), transform),
            deconstruction.element_name(),
            deconstruction.position().clone(),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            transform_expression(update.record(), transform),
            update
                .elements()
                .iter()
                .map(|element| {
                    RecordField::new(
                        element.name(),
                        transform_expression(element.expression(), transform),
                        element.position().clone(),
                    )
                })
                .collect(),
            update.position().clone(),
        )
        .into(),
        Expression::Thunk(thunk) => Thunk::new(
            thunk.type_().cloned(),
            transform_expression(thunk.expression(), transform),
            thunk.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => TypeCoercion::new(
            coercion.from().clone(),
            coercion.to().clone(),
            transform_expression(coercion.argument(), transform),
            coercion.position().clone(),
        )
        .into(),
        Expression::Variable(variable) => transform(variable),
        Expression::Boolean(_)
        | Expression::String(_)
        | Expression::None(_)
        | Expression::Number(_) => expression.clone(),
    }
}

fn transform_operation(
    operation: &Operation,
    transform: &dyn Fn(&Variable) -> Expression,
) -> Operation {
    match operation {
        Operation::Arithmetic(operation) => ArithmeticOperation::new(
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Boolean(operation) => BooleanOperation::new(
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Equality(operation) => EqualityOperation::new(
            operation.type_().cloned(),
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Not(operation) => NotOperation::new(
            transform_expression(operation.expression(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Order(operation) => OrderOperation::new(
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Try(operation) => TryOperation::new(
            operation.type_().cloned(),
            transform_expression(operation.expression(), transform),
            operation.position().clone(),
        )
        .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{DefinitionFake, ModuleFake},
        types,
    };
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_variable() {
        assert_eq!(
            transform(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake()
                    ),
                    false
                )],),
                &|variable| if variable.name() == "x" {
                    Variable::new("y", variable.position().clone()).into()
                } else {
                    variable.clone().into()
                }
            ),
            Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    Variable::new("y", Position::fake()),
                    Position::fake()
                ),
                false
            )])
        );
    }

    #[test]
    fn do_not_transform_variable_shadowed_by_argument() {
        let module = Module::empty().set_definitions(vec![Definition::fake(
            "x",
            Lambda::new(
                vec![Argument::new("x", types::None::new(Position::fake()))],
                types::None::new(Position::fake()),
                Variable::new("x", Position::fake()),
                Position::fake(),
            ),
            false,
        )]);

        assert_eq!(
            transform(&module, &|variable| if variable.name() == "x" {
                Variable::new("y", variable.position().clone()).into()
            } else {
                variable.clone().into()
            }),
            module
        );
    }

    #[test]
    fn do_not_transform_variable_shadowed_by_statement() {
        let module = Module::empty().set_definitions(vec![Definition::fake(
            "x",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                Let::new(
                    Some("x".into()),
                    None,
                    None::new(Position::fake()),
                    Variable::new("x", Position::fake()),
                    Position::fake(),
                ),
                Position::fake(),
            ),
            false,
        )]);

        assert_eq!(
            transform(&module, &|variable| if variable.name() == "x" {
                Variable::new("y", variable.position().clone()).into()
            } else {
                variable.clone().into()
            }),
            module
        );
    }

    #[test]
    fn do_not_transform_shadowed_variable_in_let() {
        let module = Module::empty().set_definitions(vec![Definition::fake(
            "x",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                Let::new(
                    Some("x".into()),
                    None,
                    None::new(Position::fake()),
                    Variable::new("x", Position::fake()),
                    Position::fake(),
                ),
                Position::fake(),
            ),
            false,
        )]);

        assert_eq!(
            transform(&module, &|variable| if variable.name() == "x" {
                Variable::new("y", variable.position().clone()).into()
            } else {
                variable.clone().into()
            }),
            module
        );
    }
}
