use crate::hir::*;
use std::collections::HashMap;

pub fn qualify(module: &Module, prefix: &str) -> Module {
    let names = module
        .definitions()
        .iter()
        .map(|definition| {
            (
                definition.name().into(),
                prefix.to_owned() + definition.name(),
            )
        })
        .collect();

    Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| qualify_definition(definition, &names))
            .collect(),
    )
}

fn qualify_definition(definition: &Definition, names: &HashMap<String, String>) -> Definition {
    Definition::new(
        names[definition.name()].clone(),
        qualify_lambda(definition.lambda(), names),
        definition.is_public(),
        definition.position().clone(),
    )
}

fn qualify_lambda(lambda: &Lambda, names: &HashMap<String, String>) -> Lambda {
    Lambda::new(
        lambda.arguments().to_vec(),
        lambda.result_type().clone(),
        qualify_block(
            lambda.body(),
            &names
                .clone()
                .into_iter()
                .filter(|(name, _)| {
                    lambda
                        .arguments()
                        .iter()
                        .all(|argument| argument.name() != name)
                })
                .collect(),
        ),
        lambda.position().clone(),
    )
}

fn qualify_block(block: &Block, names: &HashMap<String, String>) -> Block {
    let mut names = names.clone();
    let mut statements = vec![];

    for statement in block.statements() {
        statements.push(qualify_statement(statement, &names));

        if let Some(name) = statement.name() {
            names.remove(name);
        }
    }

    Block::new(statements, qualify_expression(block.expression(), &names))
}

fn qualify_statement(statement: &Statement, names: &HashMap<String, String>) -> Statement {
    Statement::new(
        statement.name().map(String::from),
        qualify_expression(statement.expression(), names),
        statement.type_().cloned(),
        statement.position().clone(),
    )
}

fn qualify_expression(expression: &Expression, names: &HashMap<String, String>) -> Expression {
    match expression {
        Expression::Call(call) => Call::new(
            qualify_expression(call.function(), names),
            call.arguments()
                .iter()
                .map(|argument| qualify_expression(argument, names))
                .collect(),
            call.function_type().cloned(),
            call.position().clone(),
        )
        .into(),
        Expression::If(if_) => If::new(
            qualify_expression(if_.condition(), names),
            qualify_block(if_.then(), names),
            qualify_block(if_.else_(), names),
            if_.result_type().cloned(),
            if_.position().clone(),
        )
        .into(),
        Expression::IfList(if_) => IfList::new(
            qualify_expression(if_.argument(), names),
            if_.first_name(),
            if_.rest_name(),
            qualify_block(
                if_.then(),
                &names
                    .clone()
                    .into_iter()
                    .filter(|(name, _)| name != if_.first_name() && name != if_.rest_name())
                    .collect(),
            ),
            qualify_block(if_.else_(), names),
            if_.result_type().cloned(),
            if_.position().clone(),
        )
        .into(),
        Expression::IfType(if_) => {
            let branch_names = names
                .clone()
                .into_iter()
                .filter(|(name, _)| name != if_.name())
                .collect();

            IfType::new(
                if_.name(),
                qualify_expression(if_.argument(), names),
                if_.argument_type().cloned(),
                if_.branches()
                    .iter()
                    .map(|branch| {
                        IfTypeBranch::new(
                            branch.type_().clone(),
                            qualify_block(branch.block(), &branch_names),
                        )
                    })
                    .collect(),
                if_.else_().map(|block| qualify_block(block, &branch_names)),
                if_.result_type().cloned(),
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => qualify_lambda(lambda, names).into(),
        Expression::List(list) => List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| match element {
                    ListElement::Multiple(element) => {
                        ListElement::Multiple(qualify_expression(element, names))
                    }
                    ListElement::Single(element) => {
                        ListElement::Single(qualify_expression(element, names))
                    }
                })
                .collect(),
            list.position().clone(),
        )
        .into(),
        Expression::Operation(operation) => qualify_operation(operation, names).into(),
        Expression::RecordConstruction(construction) => RecordConstruction::new(
            construction.type_().clone(),
            construction
                .elements()
                .iter()
                .map(|(key, element)| (key.clone(), qualify_expression(element, names)))
                .collect(),
            construction.position().clone(),
        )
        .into(),
        Expression::RecordElement(element) => RecordElement::new(
            element.type_().clone(),
            element.element_name(),
            qualify_expression(element.argument(), names),
            element.position().clone(),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            qualify_expression(update.argument(), names),
            update
                .elements()
                .iter()
                .map(|(key, element)| (key.clone(), qualify_expression(element, names)))
                .collect(),
            update.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => TypeCoercion::new(
            coercion.from().clone(),
            coercion.to().clone(),
            qualify_expression(coercion.argument(), names),
            coercion.position().clone(),
        )
        .into(),
        Expression::Variable(variable) => Variable::new(
            names
                .get(variable.name())
                .map(|string| string.as_str())
                .unwrap_or_else(|| variable.name()),
            variable.position().clone(),
        )
        .into(),
        Expression::Boolean(_)
        | Expression::String(_)
        | Expression::None(_)
        | Expression::Number(_) => expression.clone(),
    }
}

fn qualify_operation(operation: &Operation, names: &HashMap<String, String>) -> Operation {
    match operation {
        Operation::Arithmetic(operation) => ArithmeticOperation::new(
            operation.operator(),
            qualify_expression(operation.lhs(), names),
            qualify_expression(operation.rhs(), names),
            operation.position().clone(),
        )
        .into(),
        Operation::Boolean(operation) => BooleanOperation::new(
            operation.operator(),
            qualify_expression(operation.lhs(), names),
            qualify_expression(operation.rhs(), names),
            operation.position().clone(),
        )
        .into(),
        Operation::Equality(operation) => EqualityOperation::new(
            operation.type_().cloned(),
            operation.operator(),
            qualify_expression(operation.lhs(), names),
            qualify_expression(operation.rhs(), names),
            operation.position().clone(),
        )
        .into(),
        Operation::Order(operation) => OrderOperation::new(
            operation.operator(),
            qualify_expression(operation.lhs(), names),
            qualify_expression(operation.rhs(), names),
            operation.position().clone(),
        )
        .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position::Position, types};
    use pretty_assertions::assert_eq;

    #[test]
    fn qualify_definition() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::new(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::dummy()),
                            Block::new(vec![], None::new(Position::dummy())),
                            Position::dummy()
                        ),
                        false,
                        Position::dummy()
                    )]
                ),
                "foo."
            ),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "foo.x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Block::new(vec![], None::new(Position::dummy())),
                        Position::dummy()
                    ),
                    false,
                    Position::dummy()
                )]
            )
        );
    }

    #[test]
    fn qualify_variable() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::new(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::dummy()),
                            Block::new(vec![], Variable::new("x", Position::dummy())),
                            Position::dummy()
                        ),
                        false,
                        Position::dummy()
                    )]
                ),
                "foo."
            ),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "foo.x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Block::new(vec![], Variable::new("foo.x", Position::dummy())),
                        Position::dummy()
                    ),
                    false,
                    Position::dummy()
                )]
            )
        );
    }

    #[test]
    fn do_not_qualify_variable_shadowed_by_argument() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::new(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", types::None::new(Position::dummy()))],
                            types::None::new(Position::dummy()),
                            Block::new(vec![], Variable::new("x", Position::dummy())),
                            Position::dummy()
                        ),
                        false,
                        Position::dummy()
                    )]
                ),
                "foo."
            ),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "foo.x",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::dummy()))],
                        types::None::new(Position::dummy()),
                        Block::new(vec![], Variable::new("x", Position::dummy())),
                        Position::dummy()
                    ),
                    false,
                    Position::dummy()
                )]
            )
        );
    }

    #[test]
    fn do_not_qualify_variable_shadowed_by_statement() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::new(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::dummy()),
                            Block::new(
                                vec![Statement::new(
                                    Some("x".into()),
                                    None::new(Position::dummy()),
                                    None,
                                    Position::dummy(),
                                )],
                                Variable::new("x", Position::dummy())
                            ),
                            Position::dummy()
                        ),
                        false,
                        Position::dummy()
                    )]
                ),
                "foo."
            ),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "foo.x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Block::new(
                            vec![Statement::new(
                                Some("x".into()),
                                None::new(Position::dummy()),
                                None,
                                Position::dummy(),
                            )],
                            Variable::new("x", Position::dummy())
                        ),
                        Position::dummy()
                    ),
                    false,
                    Position::dummy()
                )]
            )
        );
    }
}
