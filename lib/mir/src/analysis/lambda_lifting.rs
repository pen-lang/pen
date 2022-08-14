use crate::ir::*;

struct Context {
    function_definitions: Vec<FunctionDefinition>,
}

pub fn transform(module: &Module) -> Module {
    let mut context = Context {
        function_definitions: vec![],
    };

    // TODO Is this a bug of clippy?
    #[allow(clippy::needless_collect)]
    let function_definitions = module
        .function_definitions()
        .iter()
        .map(|definition| transform_function_definition(&mut context, definition))
        .collect::<Vec<_>>();

    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.function_declarations().to_vec(),
        function_definitions
            .into_iter()
            .chain(context.function_definitions)
            .collect(),
    )
}

fn transform_function_definition(
    context: &mut Context,
    definition: &FunctionDefinition,
) -> FunctionDefinition {
    FunctionDefinition::new(
        definition.name(),
        definition.arguments().to_vec(),
        transform_expression(context, definition.body()),
        definition.result_type().clone(),
    )
}

fn transform_expression(context: &mut Context, expression: &Expression) -> Expression {
    let mut transform = |expression| transform_expression(context, expression);

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
                        transform(alternative.expression()),
                    )
                })
                .collect(),
            case.default_alternative().map(|alternative| {
                DefaultAlternative::new(alternative.name(), transform(alternative.expression()))
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
        Expression::Call(call) => Call::new(
            call.type_().clone(),
            transform(call.function()),
            call.arguments().iter().map(transform).collect(),
        )
        .into(),
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
            transform(let_.expression()),
        )
        .into(),
        Expression::LetRecursive(let_) => {
            let definition = transform_function_definition(context, let_.definition());
            let expression = transform_expression(context, let_.expression());

            if definition.environment().is_empty() {
                let name = global_closure_name(definition.name());

                context
                    .function_definitions
                    .push(FunctionDefinition::with_options(
                        &name,
                        definition.environment().to_vec(),
                        definition.arguments().to_vec(),
                        definition.body().clone(),
                        definition.result_type().clone(),
                        definition.arguments().is_empty(),
                    ));

                Let::new(
                    definition.name(),
                    definition.type_().clone(),
                    Variable::new(name),
                    expression,
                )
                .into()
            } else {
                LetRecursive::new(definition, expression).into()
            }
        }
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
            transform_expression(context, operation.operand()),
            operation.name(),
            operation.type_().clone(),
            transform_expression(context, operation.then()),
        )
        .into(),
        Expression::Variant(variant) => Variant::new(
            variant.type_().clone(),
            transform_expression(context, variant.payload()),
        )
        .into(),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => expression.clone(),
    }
}

fn global_closure_name(name: &str) -> String {
    "closure:".to_owned() + name
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::ModuleFake;
    use crate::types::{self, Type};
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_empty_module() {
        assert_eq!(transform(&Module::empty()), Module::empty());
    }

    #[test]
    fn transform_function_definition_without_closure() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::new(
            "f",
            vec![],
            42.0,
            Type::Number,
        )]);

        assert_eq!(transform(&module), module);
    }

    #[test]
    fn lift_closure_without_argument_and_free_variable() {
        let closure_name = global_closure_name("g");

        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    LetRecursive::new(
                        FunctionDefinition::new("g", vec![], 42.0, Type::Number,),
                        42.0
                    ),
                    Type::Number,
                )])
            ),
            Module::empty().set_function_definitions(vec![
                FunctionDefinition::new(
                    "f",
                    vec![],
                    Let::new(
                        "g",
                        types::Function::new(vec![], Type::Number),
                        Variable::new(&closure_name),
                        42.0
                    ),
                    Type::Number,
                ),
                FunctionDefinition::thunk(closure_name, 42.0, Type::Number)
            ])
        );
    }

    #[test]
    fn lift_closure_with_argument_and_no_free_variable() {
        let closure_name = global_closure_name("g");

        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "g",
                            vec![Argument::new("x", Type::None)],
                            42.0,
                            Type::Number,
                        ),
                        42.0
                    ),
                    Type::Number,
                )])
            ),
            Module::empty().set_function_definitions(vec![
                FunctionDefinition::new(
                    "f",
                    vec![],
                    Let::new(
                        "g",
                        types::Function::new(vec![Type::None], Type::Number),
                        Variable::new(&closure_name),
                        42.0
                    ),
                    Type::Number,
                ),
                FunctionDefinition::new(
                    closure_name,
                    vec![Argument::new("x", Type::None)],
                    42.0,
                    Type::Number,
                )
            ])
        );
    }
}
