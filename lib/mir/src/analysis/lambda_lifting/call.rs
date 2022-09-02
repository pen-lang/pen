use crate::{ir::*, types};

pub fn transform(expression: &Expression, name: &str, free_variables: &[Argument]) -> Expression {
    let transform = |expression: &_| transform(expression, name, free_variables);
    let transform_shadowed = |expression: &Expression, shadowed_name| {
        if shadowed_name == name {
            expression.clone()
        } else {
            transform(expression)
        }
    };

    match expression {
        Expression::ArithmeticOperation(operation) => ArithmeticOperation::new(
            operation.operator(),
            transform(operation.lhs()),
            transform(operation.rhs()),
        )
        .into(),
        Expression::Case(case) => Case::new(
            transform(case.argument()),
            case.alternatives()
                .iter()
                .map(|alternative| {
                    Alternative::new(
                        alternative.types().to_vec(),
                        alternative.name(),
                        transform_shadowed(alternative.expression(), alternative.name()),
                    )
                })
                .collect(),
            case.default_alternative().map(|alternative| {
                DefaultAlternative::new(
                    alternative.name(),
                    transform_shadowed(alternative.expression(), alternative.name()),
                )
            }),
        )
        .into(),
        Expression::CloneVariables(clone) => {
            CloneVariables::new(clone.variables().clone(), transform(clone.expression())).into()
        }
        Expression::ComparisonOperation(operation) => ComparisonOperation::new(
            operation.operator(),
            transform(operation.lhs()),
            transform(operation.rhs()),
        )
        .into(),
        Expression::DropVariables(drop) => {
            DropVariables::new(drop.variables().clone(), transform(drop.expression())).into()
        }
        Expression::Call(call) => {
            let function = transform(call.function());
            let arguments = call.arguments().iter().map(transform);

            if let Expression::Variable(variable) = call.function() {
                if variable.name() == name {
                    Call::new(
                        types::Function::new(
                            call.type_()
                                .arguments()
                                .iter()
                                .cloned()
                                .chain(
                                    free_variables
                                        .iter()
                                        .map(|free_variable| free_variable.type_())
                                        .cloned(),
                                )
                                .collect(),
                            call.type_().result().clone(),
                        ),
                        Variable::new(name),
                        arguments
                            .chain(
                                free_variables.iter().map(|free_variable| {
                                    Variable::new(free_variable.name()).into()
                                }),
                            )
                            .collect(),
                    )
                } else {
                    Call::new(call.type_().clone(), function, arguments.collect())
                }
            } else {
                Call::new(call.type_().clone(), function, arguments.collect())
            }
            .into()
        }
        Expression::If(if_) => If::new(
            transform(if_.condition()),
            transform(if_.then()),
            transform(if_.else_()),
        )
        .into(),
        Expression::Let(let_) => Let::new(
            let_.name(),
            let_.type_().clone(),
            transform(let_.bound_expression()),
            transform_shadowed(let_.expression(), let_.name()),
        )
        .into(),
        Expression::LetRecursive(let_) => LetRecursive::new(
            let_.definition().clone(),
            transform_shadowed(let_.expression(), let_.definition().name()),
        )
        .into(),
        Expression::Synchronize(synchronize) => Synchronize::new(
            synchronize.type_().clone(),
            transform(synchronize.expression()),
        )
        .into(),
        Expression::Record(record) => Record::new(
            record.type_().clone(),
            record.fields().iter().map(transform).collect(),
        )
        .into(),
        Expression::RecordField(field) => RecordField::new(
            field.type_().clone(),
            field.index(),
            transform(field.record()),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            transform(update.record()),
            update
                .fields()
                .iter()
                .map(|field| RecordUpdateField::new(field.index(), transform(field.expression())))
                .collect(),
        )
        .into(),
        Expression::TryOperation(operation) => TryOperation::new(
            transform(operation.operand()),
            operation.name(),
            operation.type_().clone(),
            transform_shadowed(operation.then(), operation.name()),
        )
        .into(),
        Expression::Variant(variant) => {
            Variant::new(variant.type_().clone(), transform(variant.payload())).into()
        }
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => expression.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Type;
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_call() {
        let call = Call::new(
            types::Function::new(vec![], Type::Number),
            Variable::new("f"),
            vec![],
        )
        .into();

        assert_eq!(transform(&call, "f", &[]), call);
    }

    #[test]
    fn transform_call_with_free_variable() {
        assert_eq!(
            transform(
                &Call::new(
                    types::Function::new(vec![], Type::Number),
                    Variable::new("f"),
                    vec![],
                )
                .into(),
                "f",
                &[Argument::new("x", Type::Number)]
            ),
            Call::new(
                types::Function::new(vec![Type::Number], Type::Number),
                Variable::new("f"),
                vec![Variable::new("x").into()],
            )
            .into()
        );
    }

    #[test]
    fn transform_call_in_case() {
        let function_type = types::Function::new(vec![], Type::Number);
        let expression = Case::new(
            Variant::new(function_type.clone(), 42.0),
            vec![Alternative::new(
                vec![function_type.clone().into()],
                "f",
                Call::new(function_type, Variable::new("f"), vec![]),
            )],
            None,
        )
        .into();

        assert_eq!(transform(&expression, "f", &[]), expression);
    }

    #[test]
    fn transform_call_in_case_default_alternative() {
        let function_type = types::Function::new(vec![], Type::Number);
        let expression = Case::new(
            Variant::new(function_type.clone(), 42.0),
            vec![],
            Some(DefaultAlternative::new(
                "f",
                Call::new(function_type, Variable::new("f"), vec![]),
            )),
        )
        .into();

        assert_eq!(transform(&expression, "f", &[]), expression);
    }

    #[test]
    fn transform_call_in_let() {
        let function_type = types::Function::new(vec![], Type::Number);
        let let_ = Let::new(
            "x",
            function_type.clone(),
            Variable::new("y"),
            Call::new(function_type, Variable::new("x"), vec![]),
        )
        .into();

        assert_eq!(transform(&let_, "x", &[]), let_);
    }

    #[test]
    fn transform_call_in_let_recursive() {
        let function_type = types::Function::new(vec![], Type::Number);
        let let_ = LetRecursive::new(
            FunctionDefinition::new(
                "f",
                vec![],
                Type::Number,
                Call::new(function_type.clone(), Variable::new("g"), vec![]),
            ),
            Call::new(function_type, Variable::new("g"), vec![]),
        )
        .into();

        assert_eq!(transform(&let_, "g", &[]), let_);
    }

    #[test]
    fn transform_call_in_try_operation() {
        let function_type = types::Function::new(vec![], Type::Number);
        let expression = TryOperation::new(
            42.0,
            "f",
            function_type.clone(),
            Call::new(function_type, Variable::new("f"), vec![]),
        )
        .into();

        assert_eq!(transform(&expression, "f", &[]), expression);
    }
}
