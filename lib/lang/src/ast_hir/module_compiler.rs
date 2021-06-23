use super::error::CompileError;
use crate::{ast, hir, interface, position::Position, types};
use std::collections::HashMap;

pub fn compile(
    module: &ast::Module,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
) -> Result<hir::Module, CompileError> {
    Ok(hir::Module::new(
        module_interfaces
            .values()
            .flat_map(|module_interface| {
                module_interface
                    .type_definitions()
                    .iter()
                    .map(|definition| {
                        hir::TypeDefinition::new(
                            definition.name(),
                            definition.original_name(),
                            definition.elements().to_vec(),
                            definition.is_open(),
                            definition.is_public(),
                            true,
                            definition.position().clone(),
                        )
                    })
            })
            .chain(module.type_definitions().iter().map(|definition| {
                hir::TypeDefinition::new(
                    definition.name(),
                    definition.name(),
                    definition.elements().to_vec(),
                    is_record_open(definition.elements()),
                    is_name_public(definition.name()),
                    false,
                    definition.position().clone(),
                )
            }))
            .collect(),
        module_interfaces
            .values()
            .flat_map(|module_interface| {
                module_interface.type_aliases().iter().map(|alias| {
                    hir::TypeAlias::new(
                        alias.name(),
                        alias.original_name(),
                        alias.type_().clone(),
                        alias.is_public(),
                        true,
                    )
                })
            })
            .chain(module.type_aliases().iter().map(|alias| {
                hir::TypeAlias::new(
                    alias.name(),
                    alias.name(),
                    alias.type_().clone(),
                    is_name_public(alias.name()),
                    false,
                )
            }))
            .collect(),
        module_interfaces
            .values()
            .flat_map(|interface| interface.declarations())
            .map(|declaration| {
                hir::Declaration::new(
                    declaration.name(),
                    declaration.type_().clone(),
                    declaration.position().clone(),
                )
            })
            .collect(),
        module
            .definitions()
            .iter()
            .map(|definition| compile_definition(definition))
            .collect::<Result<_, _>>()?,
    ))
}

fn compile_definition(definition: &ast::Definition) -> Result<hir::Definition, CompileError> {
    Ok(hir::Definition::new(
        definition.name(),
        definition.name(),
        compile_lambda(definition.lambda())?,
        is_name_public(definition.name()),
        definition.position().clone(),
    ))
}

fn compile_lambda(lambda: &ast::Lambda) -> Result<hir::Lambda, CompileError> {
    Ok(hir::Lambda::new(
        lambda
            .arguments()
            .iter()
            .map(|argument| hir::Argument::new(argument.name(), argument.type_().clone()))
            .collect(),
        lambda.result_type().clone(),
        compile_block(lambda.body())?,
        lambda.position().clone(),
    ))
}

fn compile_block(block: &ast::Block) -> Result<hir::Expression, CompileError> {
    let mut expression = compile_expression(block.expression())?;

    for statement in block.statements().iter().rev() {
        expression = hir::Let::new(
            statement.name().map(String::from),
            None,
            compile_expression(statement.expression())?,
            expression,
            statement.position().clone(),
        )
        .into();
    }

    Ok(expression)
}

fn compile_expression(expression: &ast::Expression) -> Result<hir::Expression, CompileError> {
    Ok(match expression {
        ast::Expression::BinaryOperation(operation) => {
            let lhs = compile_expression(operation.lhs())?;
            let rhs = compile_expression(operation.rhs())?;
            let position = operation.position().clone();

            match operation.operator() {
                ast::BinaryOperator::Add => {
                    hir::ArithmeticOperation::new(hir::ArithmeticOperator::Add, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::Subtract => hir::ArithmeticOperation::new(
                    hir::ArithmeticOperator::Subtract,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::Multiply => hir::ArithmeticOperation::new(
                    hir::ArithmeticOperator::Multiply,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::Divide => hir::ArithmeticOperation::new(
                    hir::ArithmeticOperator::Divide,
                    lhs,
                    rhs,
                    position,
                )
                .into(),

                ast::BinaryOperator::And => {
                    hir::BooleanOperation::new(hir::BooleanOperator::And, lhs, rhs, position).into()
                }
                ast::BinaryOperator::Or => {
                    hir::BooleanOperation::new(hir::BooleanOperator::Or, lhs, rhs, position).into()
                }

                ast::BinaryOperator::Equal => hir::EqualityOperation::new(
                    None,
                    hir::EqualityOperator::Equal,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::NotEqual => hir::EqualityOperation::new(
                    None,
                    hir::EqualityOperator::NotEqual,
                    lhs,
                    rhs,
                    position,
                )
                .into(),

                ast::BinaryOperator::LessThan => {
                    hir::OrderOperation::new(hir::OrderOperator::LessThan, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::LessThanOrEqual => hir::OrderOperation::new(
                    hir::OrderOperator::LessThanOrEqual,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::GreaterThan => {
                    hir::OrderOperation::new(hir::OrderOperator::GreaterThan, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::GreaterThanOrEqual => hir::OrderOperation::new(
                    hir::OrderOperator::GreaterThanOrEqual,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
            }
        }
        ast::Expression::Boolean(boolean) => {
            hir::Boolean::new(boolean.value(), boolean.position().clone()).into()
        }
        ast::Expression::Call(call) => hir::Call::new(
            compile_expression(call.function())?,
            call.arguments()
                .iter()
                .map(|argument| compile_expression(argument))
                .collect::<Result<_, _>>()?,
            None,
            call.position().clone(),
        )
        .into(),
        ast::Expression::If(if_) => compile_if(if_.branches(), if_.else_(), if_.position())?.into(),
        ast::Expression::IfList(if_) => hir::IfList::new(
            compile_expression(if_.argument())?,
            if_.first_name(),
            if_.rest_name(),
            compile_block(if_.then())?,
            compile_block(if_.else_())?,
            if_.position().clone(),
        )
        .into(),
        ast::Expression::IfType(if_) => hir::IfType::new(
            if_.name(),
            compile_expression(if_.argument())?,
            if_.branches()
                .iter()
                .map(|branch| {
                    Ok(hir::IfTypeBranch::new(
                        branch.type_().clone(),
                        compile_block(branch.block())?,
                    ))
                })
                .collect::<Result<_, _>>()?,
            None,
            if_.position().clone(),
        )
        .into(),
        ast::Expression::Lambda(lambda) => compile_lambda(lambda)?.into(),
        ast::Expression::List(list) => hir::List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| {
                    Ok(match element {
                        ast::ListElement::Multiple(element) => {
                            hir::ListElement::Multiple(compile_expression(element)?)
                        }
                        ast::ListElement::Single(element) => {
                            hir::ListElement::Single(compile_expression(element)?)
                        }
                    })
                })
                .collect::<Result<_, _>>()?,
            list.position().clone(),
        )
        .into(),
        ast::Expression::None(none) => hir::None::new(none.position().clone()).into(),
        ast::Expression::Number(number) => {
            hir::Number::new(number.value(), number.position().clone()).into()
        }
        ast::Expression::Record(record) => {
            let elements = record
                .elements()
                .iter()
                .map(|(key, element)| Ok((key.clone(), compile_expression(element)?)))
                .collect::<Result<_, _>>()?;

            if let Some(old_record) = record.record() {
                hir::RecordUpdate::new(
                    record.type_().clone(),
                    compile_expression(old_record)?,
                    elements,
                    record.position().clone(),
                )
                .into()
            } else {
                hir::RecordConstruction::new(
                    record.type_().clone(),
                    elements,
                    record.position().clone(),
                )
                .into()
            }
        }
        ast::Expression::String(string) => {
            hir::ByteString::new(string.value(), string.position().clone()).into()
        }
        ast::Expression::UnaryOperation(operation) => {
            let operand = compile_expression(operation.expression())?;

            match operation.operator() {
                ast::UnaryOperator::Not => {
                    hir::NotOperation::new(operand, operation.position().clone()).into()
                }
                ast::UnaryOperator::Try => {
                    hir::TryOperation::new(operand, operation.position().clone()).into()
                }
            }
        }
        ast::Expression::Variable(variable) => {
            hir::Variable::new(variable.name(), variable.position().clone()).into()
        }
    })
}

fn compile_if(
    branches: &[ast::IfBranch],
    else_: &ast::Block,
    position: &Position,
) -> Result<hir::If, CompileError> {
    Ok(match branches {
        [] => return Err(CompileError::TooFewBranchesInIf(position.clone())),
        [then] => hir::If::new(
            compile_expression(then.condition())?,
            compile_block(then.block())?,
            compile_block(else_)?,
            position.clone(),
        ),
        [then, ..] => hir::If::new(
            compile_expression(then.condition())?,
            compile_block(then.block())?,
            compile_if(&branches[1..], else_, position)?,
            position.clone(),
        ),
    })
}

fn is_record_open(elements: &[types::RecordElement]) -> bool {
    !elements.is_empty()
        && elements
            .iter()
            .all(|element| is_name_public(element.name()))
}

fn is_name_public(name: &str) -> bool {
    name.chars().next().unwrap().is_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn compile_empty_module() {
        assert_eq!(
            compile(
                &ast::Module::new(vec![], vec![], vec![], vec![]),
                &Default::default()
            ),
            Ok(hir::Module::new(vec![], vec![], vec![], vec![]))
        );
    }

    #[test]
    fn compile_module_with_module_interface() {
        assert_eq!(
            compile(
                &ast::Module::new(
                    vec![],
                    vec![ast::TypeDefinition::new("Foo1", vec![], Position::dummy())],
                    vec![ast::TypeAlias::new(
                        "Foo2",
                        types::None::new(Position::dummy())
                    )],
                    vec![ast::Definition::new(
                        "Foo3",
                        ast::Lambda::new(
                            vec![],
                            types::None::new(Position::dummy()),
                            ast::Block::new(vec![], ast::None::new(Position::dummy())),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )]
                ),
                &vec![(
                    ast::InternalModulePath::new(vec!["Foo".into()]).into(),
                    interface::Module::new(
                        vec![interface::TypeDefinition::without_source(
                            "Bar1",
                            vec![],
                            false,
                            true
                        )],
                        vec![interface::TypeAlias::without_source(
                            "Bar2",
                            types::None::new(Position::dummy()),
                            true,
                        )],
                        vec![interface::Declaration::without_source(
                            "Bar3",
                            types::Function::new(
                                vec![],
                                types::None::new(Position::dummy()),
                                Position::dummy()
                            ),
                            Position::dummy()
                        )]
                    )
                )]
                .into_iter()
                .collect()
            ),
            Ok(hir::Module::new(
                vec![
                    hir::TypeDefinition::without_source("Bar1", vec![], false, true, true),
                    hir::TypeDefinition::new(
                        "Foo1",
                        "Foo1",
                        vec![],
                        false,
                        true,
                        false,
                        Position::dummy()
                    )
                ],
                vec![
                    hir::TypeAlias::without_source(
                        "Bar2",
                        types::None::new(Position::dummy()),
                        true,
                        true
                    ),
                    hir::TypeAlias::new(
                        "Foo2",
                        "Foo2",
                        types::None::new(Position::dummy()),
                        true,
                        false
                    )
                ],
                vec![hir::Declaration::new(
                    "Bar3",
                    types::Function::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                )],
                vec![hir::Definition::new(
                    "Foo3",
                    "Foo3",
                    hir::Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        hir::None::new(Position::dummy()),
                        Position::dummy(),
                    ),
                    true,
                    Position::dummy()
                )]
            ))
        );
    }
}
