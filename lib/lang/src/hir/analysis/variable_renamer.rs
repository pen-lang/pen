use crate::hir::*;
use std::collections::HashMap;

pub fn rename(module: &Module, names: &HashMap<String, String>) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| rename_definition(definition, names))
            .collect(),
    )
}

fn rename_definition(definition: &Definition, names: &HashMap<String, String>) -> Definition {
    Definition::new(
        definition.name(),
        definition.original_name(),
        rename_lambda(definition.lambda(), names),
        definition.is_public(),
        definition.position().clone(),
    )
}

fn rename_lambda(lambda: &Lambda, names: &HashMap<String, String>) -> Lambda {
    Lambda::new(
        lambda.arguments().to_vec(),
        lambda.result_type().clone(),
        rename_block(
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

fn rename_block(block: &Block, names: &HashMap<String, String>) -> Block {
    let mut names = names.clone();
    let mut statements = vec![];

    for statement in block.statements() {
        statements.push(rename_statement(statement, &names));

        if let Some(name) = statement.name() {
            names.remove(name);
        }
    }

    Block::new(statements, rename_expression(block.expression(), &names))
}

fn rename_statement(statement: &Statement, names: &HashMap<String, String>) -> Statement {
    Statement::new(
        statement.name().map(String::from),
        rename_expression(statement.expression(), names),
        statement.type_().cloned(),
        statement.position().clone(),
    )
}

fn rename_expression(expression: &Expression, names: &HashMap<String, String>) -> Expression {
    match expression {
        Expression::Call(call) => Call::new(
            rename_expression(call.function(), names),
            call.arguments()
                .iter()
                .map(|argument| rename_expression(argument, names))
                .collect(),
            call.function_type().cloned(),
            call.position().clone(),
        )
        .into(),
        Expression::If(if_) => If::new(
            rename_expression(if_.condition(), names),
            rename_block(if_.then(), names),
            rename_block(if_.else_(), names),
            if_.result_type().cloned(),
            if_.position().clone(),
        )
        .into(),
        Expression::IfList(if_) => IfList::new(
            rename_expression(if_.argument(), names),
            if_.first_name(),
            if_.rest_name(),
            rename_block(
                if_.then(),
                &names
                    .clone()
                    .into_iter()
                    .filter(|(name, _)| name != if_.first_name() && name != if_.rest_name())
                    .collect(),
            ),
            rename_block(if_.else_(), names),
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
                rename_expression(if_.argument(), names),
                if_.argument_type().cloned(),
                if_.branches()
                    .iter()
                    .map(|branch| {
                        IfTypeBranch::new(
                            branch.type_().clone(),
                            rename_block(branch.block(), &branch_names),
                        )
                    })
                    .collect(),
                if_.else_().map(|block| rename_block(block, &branch_names)),
                if_.result_type().cloned(),
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => rename_lambda(lambda, names).into(),
        Expression::Let(let_) => Let::new(
            let_.name().map(String::from),
            let_.type_().cloned(),
            rename_expression(let_.bound_expression(), names),
            rename_expression(
                let_.expression(),
                &names
                    .clone()
                    .into_iter()
                    .filter(|(name, _)| Some(name.as_str()) != let_.name())
                    .collect(),
            ),
            let_.position().clone(),
        )
        .into(),
        Expression::List(list) => List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| match element {
                    ListElement::Multiple(element) => {
                        ListElement::Multiple(rename_expression(element, names))
                    }
                    ListElement::Single(element) => {
                        ListElement::Single(rename_expression(element, names))
                    }
                })
                .collect(),
            list.position().clone(),
        )
        .into(),
        Expression::Operation(operation) => rename_operation(operation, names).into(),
        Expression::RecordConstruction(construction) => RecordConstruction::new(
            construction.type_().clone(),
            construction
                .elements()
                .iter()
                .map(|(key, element)| (key.clone(), rename_expression(element, names)))
                .collect(),
            construction.position().clone(),
        )
        .into(),
        Expression::RecordElement(element) => RecordElement::new(
            element.type_().cloned(),
            rename_expression(element.record(), names),
            element.element_name(),
            element.position().clone(),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            rename_expression(update.record(), names),
            update
                .elements()
                .iter()
                .map(|(key, element)| (key.clone(), rename_expression(element, names)))
                .collect(),
            update.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => TypeCoercion::new(
            coercion.from().clone(),
            coercion.to().clone(),
            rename_expression(coercion.argument(), names),
            coercion.position().clone(),
        )
        .into(),
        Expression::Variable(variable) => Variable::new(
            names
                .get(variable.name())
                .map(|string| string.as_str())
                .unwrap_or_else(|| variable.name()),
            variable.type_().cloned(),
            variable.position().clone(),
        )
        .into(),
        Expression::Boolean(_)
        | Expression::String(_)
        | Expression::None(_)
        | Expression::Number(_) => expression.clone(),
    }
}

fn rename_operation(operation: &Operation, names: &HashMap<String, String>) -> Operation {
    match operation {
        Operation::Arithmetic(operation) => ArithmeticOperation::new(
            operation.operator(),
            rename_expression(operation.lhs(), names),
            rename_expression(operation.rhs(), names),
            operation.position().clone(),
        )
        .into(),
        Operation::Boolean(operation) => BooleanOperation::new(
            operation.operator(),
            rename_expression(operation.lhs(), names),
            rename_expression(operation.rhs(), names),
            operation.position().clone(),
        )
        .into(),
        Operation::Equality(operation) => EqualityOperation::new(
            operation.type_().cloned(),
            operation.operator(),
            rename_expression(operation.lhs(), names),
            rename_expression(operation.rhs(), names),
            operation.position().clone(),
        )
        .into(),
        Operation::Not(operation) => NotOperation::new(
            rename_expression(operation.expression(), names),
            operation.position().clone(),
        )
        .into(),
        Operation::Order(operation) => OrderOperation::new(
            operation.operator(),
            rename_expression(operation.lhs(), names),
            rename_expression(operation.rhs(), names),
            operation.position().clone(),
        )
        .into(),
        Operation::Try(operation) => TryOperation::new(
            rename_expression(operation.expression(), names),
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
    fn rename_variable() {
        assert_eq!(
            rename(
                &Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::dummy()),
                            Block::new(vec![], Variable::new("x", None, Position::dummy())),
                            Position::dummy()
                        ),
                        false
                    )]
                ),
                &vec![("x".into(), "foo.x".into())].into_iter().collect()
            ),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Block::new(vec![], Variable::new("foo.x", None, Position::dummy())),
                        Position::dummy()
                    ),
                    false
                )]
            )
        );
    }

    #[test]
    fn do_not_rename_variable_shadowed_by_argument() {
        let module = Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![Argument::new("x", types::None::new(Position::dummy()))],
                    types::None::new(Position::dummy()),
                    Block::new(vec![], Variable::new("x", None, Position::dummy())),
                    Position::dummy(),
                ),
                false,
            )],
        );

        assert_eq!(
            rename(
                &module,
                &vec![("x".into(), "foo.x".into())].into_iter().collect()
            ),
            module
        );
    }

    #[test]
    fn do_not_rename_variable_shadowed_by_statement() {
        assert_eq!(
            rename(
                &Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
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
                                Variable::new("x", None, Position::dummy())
                            ),
                            Position::dummy()
                        ),
                        false
                    )]
                ),
                &vec![("x".into(), "foo.x".into())].into_iter().collect()
            ),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
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
                            Variable::new("x", None, Position::dummy())
                        ),
                        Position::dummy()
                    ),
                    false
                )]
            )
        );
    }

    #[test]
    fn do_not_rename_shadowed_variable_in_let() {
        let module = Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    Block::new(
                        vec![],
                        Let::new(
                            Some("x".into()),
                            None,
                            None::new(Position::dummy()),
                            Variable::new("x", None, Position::dummy()),
                            Position::dummy(),
                        ),
                    ),
                    Position::dummy(),
                ),
                false,
            )],
        );

        assert_eq!(
            rename(
                &module,
                &vec![("x".into(), "foo.x".into())].into_iter().collect()
            ),
            module
        );
    }
}
