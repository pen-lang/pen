use super::error::CompileError;
use crate::{ast, hir, position::Position, types};

pub fn compile(module: &ast::Module) -> Result<hir::Module, CompileError> {
    Ok(hir::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|definition| {
                hir::TypeDefinition::new(
                    definition.name(),
                    definition.name(),
                    definition.elements().to_vec(),
                    is_record_open(definition.elements()),
                    is_name_public(definition.name()),
                    false,
                    definition.position().clone(),
                )
            })
            .collect(),
        module
            .type_aliases()
            .iter()
            .map(|alias| {
                hir::TypeAlias::new(
                    alias.name(),
                    alias.name(),
                    alias.type_().clone(),
                    is_name_public(alias.name()),
                    false,
                )
            })
            .collect(),
        module
            .foreign_imports()
            .iter()
            .map(|import| {
                hir::ForeignDeclaration::new(
                    import.name(),
                    import.foreign_name(),
                    match import.calling_convention() {
                        ast::CallingConvention::C => hir::CallingConvention::C,
                        ast::CallingConvention::Native => hir::CallingConvention::Native,
                    },
                    import.type_().clone(),
                    import.position().clone(),
                )
            })
            .collect(),
        vec![],
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
        ast::Expression::RecordDeconstruction(operation) => hir::RecordDeconstruction::new(
            None,
            compile_expression(operation.expression())?,
            operation.name(),
            operation.position().clone(),
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
            if_.else_()
                .map(|block| {
                    Ok(hir::ElseBranch::new(
                        None,
                        compile_block(block)?,
                        block.position().clone(),
                    ))
                })
                .transpose()?,
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
                .map(|element| {
                    Ok(hir::RecordElement::new(
                        element.name(),
                        compile_expression(element.expression())?,
                        element.position().clone(),
                    ))
                })
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
            compile(&ast::Module::new(vec![], vec![], vec![], vec![], vec![])),
            Ok(hir::Module::new(vec![], vec![], vec![], vec![], vec![],))
        );
    }

    #[test]
    fn compile_module() {
        assert_eq!(
            compile(&ast::Module::new(
                vec![],
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
                        ast::Block::new(
                            vec![],
                            ast::None::new(Position::dummy()),
                            Position::dummy()
                        ),
                        Position::dummy(),
                    ),
                    Position::dummy(),
                )]
            )),
            Ok(hir::Module::empty()
                .set_type_definitions(vec![hir::TypeDefinition::new(
                    "Foo1",
                    "Foo1",
                    vec![],
                    false,
                    true,
                    false,
                    Position::dummy()
                )])
                .set_type_aliases(vec![hir::TypeAlias::new(
                    "Foo2",
                    "Foo2",
                    types::None::new(Position::dummy()),
                    true,
                    false
                )])
                .set_definitions(vec![hir::Definition::new(
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
                )]))
        );
    }
}
