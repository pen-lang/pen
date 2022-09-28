use crate::{ir::*, types::Type};
use fnv::FnvHashSet;

pub fn collect(module: &Module) -> FnvHashSet<Type> {
    let mut types = FnvHashSet::default();

    for definition in module.function_definitions() {
        collect_from_function_definition(definition.definition(), &mut types);
    }

    types
}

fn collect_from_function_definition(definition: &FunctionDefinition, types: &mut FnvHashSet<Type>) {
    collect_from_expression(definition.body(), types)
}

fn collect_from_expression(expression: &Expression, types: &mut FnvHashSet<Type>) {
    match expression {
        Expression::ArithmeticOperation(operation) => {
            for expression in [operation.lhs(), operation.rhs()] {
                collect_from_expression(expression, types);
            }
        }
        Expression::Case(case) => {
            collect_from_expression(case.argument(), types);

            for alternative in case.alternatives() {
                types.extend(alternative.types().iter().cloned());

                collect_from_expression(alternative.expression(), types);

                if let Some(alternative) = case.default_alternative() {
                    collect_from_expression(alternative.expression(), types);
                }
            }
        }
        Expression::CloneVariables(clone) => collect_from_expression(clone.expression(), types),
        Expression::ComparisonOperation(operation) => {
            for expression in [operation.lhs(), operation.rhs()] {
                collect_from_expression(expression, types);
            }
        }
        Expression::DropVariables(drop) => collect_from_expression(drop.expression(), types),
        Expression::Call(call) => {
            collect_from_expression(call.function(), types);

            for argument in call.arguments() {
                collect_from_expression(argument, types);
            }
        }
        Expression::If(if_) => {
            for expression in [if_.condition(), if_.then(), if_.else_()] {
                collect_from_expression(expression, types);
            }
        }
        Expression::Let(let_) => {
            for expression in [let_.bound_expression(), let_.expression()] {
                collect_from_expression(expression, types);
            }
        }
        Expression::LetRecursive(let_) => {
            collect_from_function_definition(let_.definition(), types);
            collect_from_expression(let_.expression(), types);
        }
        Expression::Synchronize(synchronize) => {
            collect_from_expression(synchronize.expression(), types)
        }
        Expression::Record(record) => {
            for field in record.fields() {
                collect_from_expression(field, types);
            }
        }
        Expression::RecordField(field) => collect_from_expression(field.record(), types),
        Expression::RecordUpdate(update) => {
            collect_from_expression(update.record(), types);

            for field in update.fields() {
                collect_from_expression(field.expression(), types);
            }
        }
        Expression::StringConcatenation(concatenation) => {
            for operand in concatenation.operands() {
                collect_from_expression(operand, types);
            }
        }
        Expression::TryOperation(operation) => {
            types.insert(operation.type_().clone());

            collect_from_expression(operation.operand(), types);
            collect_from_expression(operation.then(), types);
        }
        Expression::TypeInformationFunction(information) => {
            collect_from_expression(information.variant(), types)
        }
        Expression::Variant(variant) => {
            types.insert(variant.type_().clone());

            collect_from_expression(variant.payload(), types);
        }
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test::ModuleFake, types::Type};

    #[test]
    fn collect_from_case_argument() {
        assert_eq!(
            collect(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::None,
                    Case::new(Variant::new(Type::Number, Variable::new("x")), vec![], None)
                )])
            ),
            [Type::Number].into_iter().collect()
        );
    }

    #[test]
    fn collect_from_try_operation_operand() {
        assert_eq!(
            collect(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    Type::None,
                    TryOperation::new(
                        Variable::new("x"),
                        "error",
                        Type::Number,
                        Variable::new("error"),
                    )
                )],)
            ),
            [Type::Number].into_iter().collect()
        );
    }

    #[test]
    fn collect_from_try_operation_then_expression() {
        assert_eq!(
            collect(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    Type::None,
                    TryOperation::new(
                        Variable::new("x"),
                        "error",
                        Type::Number,
                        Variant::new(Type::Boolean, Variable::new("error")),
                    )
                )],)
            ),
            [Type::Boolean, Type::Number].into_iter().collect()
        );
    }
}
