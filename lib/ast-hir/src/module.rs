use super::error::CompileError;
use crate::{number, string, type_};
use hir::{ir, types};
use position::Position;

pub fn compile(module: &ast::Module) -> Result<ir::Module, CompileError> {
    Ok(ir::Module::new(
        module
            .type_definitions()
            .iter()
            .filter_map(|definition| match definition {
                ast::TypeDefinition::RecordDefinition(definition) => Some(ir::TypeDefinition::new(
                    definition.name(),
                    definition.name(),
                    definition
                        .fields()
                        .iter()
                        .map(|field| {
                            types::RecordField::new(field.name(), type_::compile(field.type_()))
                        })
                        .collect(),
                    ast::analysis::is_record_open(definition),
                    ast::analysis::is_name_public(definition.name()),
                    false,
                    definition.position().clone(),
                )),
                ast::TypeDefinition::TypeAlias(_) => None,
            })
            .collect(),
        module
            .type_definitions()
            .iter()
            .filter_map(|definition| match definition {
                ast::TypeDefinition::RecordDefinition(_) => None,
                ast::TypeDefinition::TypeAlias(alias) => Some(ir::TypeAlias::new(
                    alias.name(),
                    alias.name(),
                    type_::compile(alias.type_()),
                    ast::analysis::is_name_public(alias.name()),
                    false,
                    alias.position().clone(),
                )),
            })
            .collect(),
        module
            .foreign_imports()
            .iter()
            .map(|import| {
                ir::ForeignDeclaration::new(
                    import.name(),
                    import.name(),
                    compile_calling_convention(import.calling_convention()),
                    type_::compile(import.type_()),
                    import.position().clone(),
                )
            })
            .collect(),
        vec![],
        module
            .function_definitions()
            .iter()
            .map(compile_function_definition)
            .collect::<Result<_, _>>()?,
        module.position().clone(),
    ))
}

fn compile_function_definition(
    definition: &ast::FunctionDefinition,
) -> Result<ir::FunctionDefinition, CompileError> {
    Ok(ir::FunctionDefinition::new(
        definition.name(),
        definition.name(),
        compile_lambda(definition.lambda())?,
        definition.foreign_export().map(|export| {
            ir::ForeignDefinitionConfiguration::new(compile_calling_convention(
                export.calling_convention(),
            ))
        }),
        ast::analysis::is_name_public(definition.name()),
        definition.position().clone(),
    ))
}

fn compile_calling_convention(calling_convention: ast::CallingConvention) -> ir::CallingConvention {
    match calling_convention {
        ast::CallingConvention::C => ir::CallingConvention::C,
        ast::CallingConvention::Native => ir::CallingConvention::Native,
    }
}

fn compile_lambda(lambda: &ast::Lambda) -> Result<ir::Lambda, CompileError> {
    Ok(ir::Lambda::new(
        lambda
            .arguments()
            .iter()
            .map(|argument| ir::Argument::new(argument.name(), type_::compile(argument.type_())))
            .collect(),
        type_::compile(lambda.result_type()),
        compile_block(lambda.body())?,
        lambda.position().clone(),
    ))
}

fn compile_block(block: &ast::Block) -> Result<ir::Expression, CompileError> {
    let mut expression = compile_expression(block.expression())?;

    for statement in block.statements().iter().rev() {
        expression = ir::Let::new(
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

fn compile_expression(expression: &ast::Expression) -> Result<ir::Expression, CompileError> {
    Ok(match expression {
        ast::Expression::BinaryOperation(operation) => {
            let lhs = compile_expression(operation.lhs())?;
            let rhs = compile_expression(operation.rhs())?;
            let position = operation.position().clone();

            match operation.operator() {
                ast::BinaryOperator::Add => {
                    ir::AdditionOperation::new(None, lhs, rhs, position).into()
                }
                ast::BinaryOperator::Subtract => ir::ArithmeticOperation::new(
                    ir::ArithmeticOperator::Subtract,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::Multiply => ir::ArithmeticOperation::new(
                    ir::ArithmeticOperator::Multiply,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::Divide => {
                    ir::ArithmeticOperation::new(ir::ArithmeticOperator::Divide, lhs, rhs, position)
                        .into()
                }

                ast::BinaryOperator::And => {
                    ir::BooleanOperation::new(ir::BooleanOperator::And, lhs, rhs, position).into()
                }
                ast::BinaryOperator::Or => {
                    ir::BooleanOperation::new(ir::BooleanOperator::Or, lhs, rhs, position).into()
                }

                ast::BinaryOperator::Equal => ir::EqualityOperation::new(
                    None,
                    ir::EqualityOperator::Equal,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::NotEqual => ir::EqualityOperation::new(
                    None,
                    ir::EqualityOperator::NotEqual,
                    lhs,
                    rhs,
                    position,
                )
                .into(),

                ast::BinaryOperator::LessThan => {
                    ir::OrderOperation::new(ir::OrderOperator::LessThan, lhs, rhs, position).into()
                }
                ast::BinaryOperator::LessThanOrEqual => {
                    ir::OrderOperation::new(ir::OrderOperator::LessThanOrEqual, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::GreaterThan => {
                    ir::OrderOperation::new(ir::OrderOperator::GreaterThan, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::GreaterThanOrEqual => ir::OrderOperation::new(
                    ir::OrderOperator::GreaterThanOrEqual,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
            }
        }
        ast::Expression::Call(call) => ir::Call::new(
            None,
            compile_expression(call.function())?,
            call.arguments()
                .iter()
                .map(compile_expression)
                .collect::<Result<Vec<_>, _>>()?,
            call.position().clone(),
        )
        .into(),
        ast::Expression::RecordDeconstruction(operation) => ir::RecordDeconstruction::new(
            None,
            compile_expression(operation.expression())?,
            operation.name(),
            operation.position().clone(),
        )
        .into(),
        ast::Expression::If(if_) => compile_if(if_.branches(), if_.else_(), if_.position())?.into(),
        ast::Expression::IfList(if_) => ir::IfList::new(
            None,
            compile_expression(if_.list())?,
            if_.first_name(),
            if_.rest_name(),
            compile_block(if_.then())?,
            compile_block(if_.else_())?,
            if_.position().clone(),
        )
        .into(),
        ast::Expression::IfMap(if_) => ir::IfMap::new(
            None,
            None,
            if_.name(),
            compile_expression(if_.map())?,
            compile_expression(if_.key())?,
            compile_block(if_.then())?,
            compile_block(if_.else_())?,
            if_.position().clone(),
        )
        .into(),
        ast::Expression::IfType(if_) => ir::IfType::new(
            if_.name(),
            compile_expression(if_.argument())?,
            if_.branches()
                .iter()
                .map(|branch| {
                    Ok(ir::IfTypeBranch::new(
                        type_::compile(branch.type_()),
                        compile_block(branch.block())?,
                    ))
                })
                .collect::<Result<_, _>>()?,
            if_.else_()
                .map(|block| {
                    Ok(ir::ElseBranch::new(
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
        ast::Expression::List(list) => ir::List::new(
            type_::compile(list.type_()),
            list.elements()
                .iter()
                .map(|element| {
                    Ok(match element {
                        ast::ListElement::Multiple(element) => {
                            ir::ListElement::Multiple(compile_expression(element)?)
                        }
                        ast::ListElement::Single(element) => {
                            ir::ListElement::Single(compile_expression(element)?)
                        }
                    })
                })
                .collect::<Result<_, _>>()?,
            list.position().clone(),
        )
        .into(),
        ast::Expression::ListComprehension(comprehension) => ir::ListComprehension::new(
            type_::compile(comprehension.type_()),
            compile_expression(comprehension.element())?,
            vec![ir::ListComprehensionBranch::new(
                None,
                comprehension.primary_name(),
                comprehension.secondary_name().map(String::from),
                compile_expression(comprehension.iteratee())?,
                comprehension.position().clone(),
            )],
            comprehension.position().clone(),
        )
        .into(),
        ast::Expression::Map(map) => ir::Map::new(
            type_::compile(map.key_type()),
            type_::compile(map.value_type()),
            map.elements()
                .iter()
                .map(|element| {
                    Ok(match element {
                        ast::MapElement::Insertion(entry) => {
                            ir::MapElement::Insertion(ir::MapEntry::new(
                                compile_expression(entry.key())?,
                                compile_expression(entry.value())?,
                                entry.position().clone(),
                            ))
                        }
                        ast::MapElement::Map(element) => {
                            ir::MapElement::Map(compile_expression(element)?)
                        }
                    })
                })
                .collect::<Result<_, _>>()?,
            map.position().clone(),
        )
        .into(),
        ast::Expression::Number(number) => {
            ir::Number::new(number::compile(number)?, number.position().clone()).into()
        }
        ast::Expression::Record(record) => {
            let type_ = types::Reference::new(record.type_name(), record.position().clone());
            let fields = record
                .fields()
                .iter()
                .map(|field| {
                    Ok(ir::RecordField::new(
                        field.name(),
                        compile_expression(field.expression())?,
                        field.position().clone(),
                    ))
                })
                .collect::<Result<_, _>>()?;

            if let Some(old_record) = record.record() {
                ir::RecordUpdate::new(
                    type_,
                    compile_expression(old_record)?,
                    fields,
                    record.position().clone(),
                )
                .into()
            } else {
                ir::RecordConstruction::new(type_, fields, record.position().clone()).into()
            }
        }
        ast::Expression::String(string) => {
            ir::ByteString::new(string::compile(string.value()), string.position().clone()).into()
        }
        ast::Expression::UnaryOperation(operation) => {
            let operand = compile_expression(operation.expression())?;

            match operation.operator() {
                ast::UnaryOperator::Not => {
                    ir::NotOperation::new(operand, operation.position().clone()).into()
                }
                ast::UnaryOperator::Try => {
                    ir::TryOperation::new(None, operand, operation.position().clone()).into()
                }
            }
        }
        ast::Expression::Variable(variable) => {
            ir::Variable::new(variable.name(), variable.position().clone()).into()
        }
    })
}

fn compile_if(
    branches: &[ast::IfBranch],
    else_: &ast::Block,
    position: &Position,
) -> Result<ir::If, CompileError> {
    Ok(match branches {
        [] => return Err(CompileError::TooFewBranchesInIf(position.clone())),
        [then] => ir::If::new(
            compile_expression(then.condition())?,
            compile_block(then.block())?,
            compile_block(else_)?,
            position.clone(),
        ),
        [then, ..] => ir::If::new(
            compile_expression(then.condition())?,
            compile_block(then.block())?,
            compile_if(&branches[1..], else_, position)?,
            position.clone(),
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::test::ModuleFake;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    #[test]
    fn compile_empty_module() {
        assert_eq!(
            compile(&ast::Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            Ok(ir::Module::empty())
        );
    }

    #[test]
    fn compile_module() {
        assert_eq!(
            compile(&ast::Module::new(
                vec![],
                vec![],
                vec![
                    ast::RecordDefinition::new("Foo1", vec![], Position::fake()).into(),
                    ast::TypeAlias::new(
                        "Foo2",
                        ast::types::Reference::new("none", Position::fake()),
                        Position::fake(),
                    )
                    .into()
                ],
                vec![ast::FunctionDefinition::new(
                    "Foo3",
                    ast::Lambda::new(
                        vec![],
                        ast::types::Reference::new("none", Position::fake()),
                        ast::Block::new(
                            vec![],
                            ast::Variable::new("none", Position::fake()),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    None,
                    Position::fake(),
                )],
                Position::fake(),
            )),
            Ok(ir::Module::empty()
                .set_type_definitions(vec![ir::TypeDefinition::new(
                    "Foo1",
                    "Foo1",
                    vec![],
                    true,
                    true,
                    false,
                    Position::fake()
                )])
                .set_type_aliases(vec![ir::TypeAlias::new(
                    "Foo2",
                    "Foo2",
                    types::Reference::new("none", Position::fake()),
                    true,
                    false,
                    Position::fake()
                )])
                .set_function_definitions(vec![ir::FunctionDefinition::new(
                    "Foo3",
                    "Foo3",
                    ir::Lambda::new(
                        vec![],
                        types::Reference::new("none", Position::fake()),
                        ir::Variable::new("none", Position::fake()),
                        Position::fake(),
                    ),
                    None,
                    true,
                    Position::fake()
                )]))
        );
    }
}
