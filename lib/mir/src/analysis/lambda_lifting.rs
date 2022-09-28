mod box_;
mod call;
mod context;

use self::context::Context;
use crate::{ir::*, types};
use fnv::FnvHashMap;

pub fn transform(module: &Module) -> Module {
    let mut context = Context::new();

    // TODO Is this a bug of clippy?
    #[allow(clippy::needless_collect)]
    let function_definitions = module
        .function_definitions()
        .iter()
        .map(|definition| {
            GlobalFunctionDefinition::new(
                transform_function_definition(&mut context, definition.definition()),
                definition.is_public(),
            )
        })
        .collect::<Vec<_>>();

    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.function_declarations().to_vec(),
        function_definitions
            .into_iter()
            .chain(context.into_function_definitions())
            .collect(),
        module.type_information().clone(),
    )
}

fn transform_function_definition(
    context: &mut Context,
    definition: &FunctionDefinition,
) -> FunctionDefinition {
    FunctionDefinition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        definition.result_type().clone(),
        transform_expression(context, definition.body()),
        definition.is_thunk(),
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
                let name = context.add_function_definition(definition.clone());

                Let::new(
                    definition.name(),
                    definition.type_().clone(),
                    Variable::new(name),
                    expression,
                )
                .into()
            } else if !definition.is_thunk()
                && !box_::is_boxed(definition.body(), definition.name())
                && !box_::is_boxed(&expression, definition.name())
            {
                let free_variable_names = rename_free_variables(context, definition.environment());
                let renamed_environment = definition
                    .environment()
                    .iter()
                    .map(|free_variable| {
                        Argument::new(
                            &free_variable_names[free_variable.name()],
                            free_variable.type_().clone(),
                        )
                    })
                    .collect::<Vec<_>>();
                let transform = |expression| {
                    save_free_variables(
                        definition.environment(),
                        &free_variable_names,
                        &call::transform(expression, definition.name(), &renamed_environment),
                    )
                };

                let arguments = definition
                    .arguments()
                    .iter()
                    .cloned()
                    .chain(definition.environment().iter().cloned())
                    .collect::<Vec<_>>();
                let function_name =
                    context.add_function_definition(FunctionDefinition::with_options(
                        definition.name(),
                        vec![],
                        arguments.clone(),
                        definition.result_type().clone(),
                        transform(definition.body()),
                        definition.is_thunk(),
                    ));

                Let::new(
                    definition.name(),
                    types::Function::new(
                        arguments
                            .iter()
                            .map(|argument| argument.type_())
                            .cloned()
                            .collect(),
                        definition.type_().result().clone(),
                    ),
                    Variable::new(function_name),
                    transform(&expression),
                )
                .into()
            } else {
                LetRecursive::new(definition, expression).into()
            }
        }
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
        Expression::StringConcatenation(concatenation) => {
            StringConcatenation::new(concatenation.operands().iter().map(transform).collect())
                .into()
        }
        Expression::Synchronize(synchronize) => Synchronize::new(
            synchronize.type_().clone(),
            transform(synchronize.expression()),
        )
        .into(),
        Expression::TryOperation(operation) => TryOperation::new(
            transform(operation.operand()),
            operation.name(),
            operation.type_().clone(),
            transform(operation.then()),
        )
        .into(),
        Expression::TypeInformation(information) => {
            TypeInformationFunction::new(information.index(), transform(information.variant()))
                .into()
        }
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

fn rename_free_variables<'a>(
    context: &mut Context,
    environment: &'a [Argument],
) -> FnvHashMap<&'a str, String> {
    environment
        .iter()
        .map(|free_variable| {
            (
                free_variable.name(),
                context.rename_free_variable(free_variable.name()),
            )
        })
        .collect()
}

fn save_free_variables(
    environment: &[Argument],
    names: &FnvHashMap<&str, String>,
    expression: &Expression,
) -> Expression {
    match environment {
        [] => expression.clone(),
        [free_variable, ..] => Let::new(
            &names[free_variable.name()],
            free_variable.type_().clone(),
            Variable::new(free_variable.name()),
            save_free_variables(&environment[1..], names, expression),
        )
        .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{FunctionDefinitionFake, ModuleFake},
        types::{self, Type},
    };
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
            Type::Number,
            42.0,
        )]);

        assert_eq!(transform(&module), module);
    }

    #[test]
    fn lift_closure_without_argument_and_free_variable() {
        let function_type = types::Function::new(vec![], Type::Number);

        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::new("g", vec![], Type::Number, 42.0),
                        42.0
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![
                FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type.clone(),
                        Variable::new("mir:lift:0:g"),
                        42.0
                    )
                ),
                FunctionDefinition::new(
                    "mir:lift:0:g",
                    vec![],
                    Type::Number,
                    Let::new("g", function_type, Variable::new("mir:lift:0:g"), 42.0)
                )
            ])
        );
    }

    #[test]
    fn lift_closure_with_argument_and_no_free_variable() {
        let function_type = types::Function::new(vec![Type::None], Type::Number);

        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "g",
                            vec![Argument::new("x", Type::None)],
                            Type::Number,
                            42.0
                        ),
                        42.0
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![
                FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type.clone(),
                        Variable::new("mir:lift:0:g"),
                        42.0
                    )
                ),
                FunctionDefinition::new(
                    "mir:lift:0:g",
                    vec![Argument::new("x", Type::None)],
                    Type::Number,
                    Let::new("g", function_type, Variable::new("mir:lift:0:g"), 42.0)
                )
            ])
        );
    }

    #[test]
    fn lift_closure_with_free_variable_used_in_body() {
        let function_type = types::Function::new(vec![Type::Number], Type::Number);

        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::with_options(
                            "g",
                            vec![Argument::new("x", Type::Number)],
                            vec![],
                            Type::Number,
                            Variable::new("x"),
                            false,
                        ),
                        42.0,
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![
                FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type.clone(),
                        Variable::new("mir:lift:0:g"),
                        Let::new("fv:x:0", Type::Number, Variable::new("x"), 42.0)
                    )
                ),
                FunctionDefinition::with_options(
                    "mir:lift:0:g",
                    vec![],
                    vec![Argument::new("x", Type::Number)],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type,
                        Variable::new("mir:lift:0:g"),
                        Let::new(
                            "fv:x:0",
                            Type::Number,
                            Variable::new("x"),
                            Variable::new("x")
                        )
                    ),
                    false,
                )
            ])
        );
    }
    #[test]
    fn lift_closure_with_free_variable() {
        let function_type = types::Function::new(vec![Type::None], Type::Number);

        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::with_options(
                            "g",
                            vec![Argument::new("x", Type::None)],
                            vec![],
                            Type::Number,
                            42.0,
                            false,
                        ),
                        42.0,
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![
                FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type.clone(),
                        Variable::new("mir:lift:0:g"),
                        Let::new("fv:x:0", Type::None, Variable::new("x"), 42.0)
                    )
                ),
                FunctionDefinition::with_options(
                    "mir:lift:0:g",
                    vec![],
                    vec![Argument::new("x", Type::None)],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type,
                        Variable::new("mir:lift:0:g"),
                        Let::new("fv:x:0", Type::None, Variable::new("x"), 42.0)
                    ),
                    false,
                )
            ])
        );
    }

    #[test]
    fn lift_closure_with_free_variable_with_call() {
        let function_type = types::Function::new(vec![Type::None], Type::Number);

        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::with_options(
                            "g",
                            vec![Argument::new("x", Type::None)],
                            vec![],
                            Type::Number,
                            42.0,
                            false,
                        ),
                        Call::new(
                            types::Function::new(vec![], Type::Number),
                            Variable::new("g"),
                            vec![]
                        ),
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![
                FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type.clone(),
                        Variable::new("mir:lift:0:g"),
                        Let::new(
                            "fv:x:0",
                            Type::None,
                            Variable::new("x"),
                            Call::new(
                                types::Function::new(vec![Type::None], Type::Number),
                                Variable::new("g"),
                                vec![Variable::new("fv:x:0").into()]
                            ),
                        )
                    )
                ),
                FunctionDefinition::with_options(
                    "mir:lift:0:g",
                    vec![],
                    vec![Argument::new("x", Type::None)],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type,
                        Variable::new("mir:lift:0:g"),
                        Let::new("fv:x:0", Type::None, Variable::new("x"), 42.0)
                    ),
                    false,
                )
            ])
        );
    }

    #[test]
    fn lift_recursive_closure_without_free_variable() {
        let function_type = types::Function::new(vec![Type::None], Type::Number);

        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "g",
                            vec![Argument::new("x", Type::None)],
                            Type::Number,
                            Call::new(
                                function_type.clone(),
                                Variable::new("g"),
                                vec![Variable::new("x").into()]
                            )
                        ),
                        42.0
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![
                FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type.clone(),
                        Variable::new("mir:lift:0:g"),
                        42.0
                    )
                ),
                FunctionDefinition::new(
                    "mir:lift:0:g",
                    vec![Argument::new("x", Type::None)],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type.clone(),
                        Variable::new("mir:lift:0:g"),
                        Call::new(
                            function_type,
                            Variable::new("g"),
                            vec![Variable::new("x").into()]
                        )
                    )
                )
            ])
        );
    }

    #[test]
    fn lift_recursive_closure_with_free_variable() {
        let function_type = types::Function::new(vec![Type::None], Type::Number);

        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::with_options(
                            "g",
                            vec![Argument::new("x", Type::None)],
                            vec![],
                            Type::Number,
                            Call::new(
                                types::Function::new(vec![], Type::Number),
                                Variable::new("g"),
                                vec![]
                            ),
                            false,
                        ),
                        42.0
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![
                FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type.clone(),
                        Variable::new("mir:lift:0:g"),
                        Let::new("fv:x:0", Type::None, Variable::new("x"), 42.0)
                    )
                ),
                FunctionDefinition::with_options(
                    "mir:lift:0:g",
                    vec![],
                    vec![Argument::new("x", Type::None)],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type.clone(),
                        Variable::new("mir:lift:0:g"),
                        Let::new(
                            "fv:x:0",
                            Type::None,
                            Variable::new("x"),
                            Call::new(
                                function_type,
                                Variable::new("g"),
                                vec![Variable::new("fv:x:0").into()]
                            )
                        )
                    ),
                    false,
                )
            ])
        );
    }

    #[test]
    fn lift_thunk_without_free_variable() {
        let function_type = types::Function::new(vec![], Type::Number);

        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    LetRecursive::new(FunctionDefinition::thunk("g", Type::Number, 42.0), 42.0)
                )])
            ),
            Module::empty().set_function_definitions(vec![
                FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "g",
                        function_type.clone(),
                        Variable::new("mir:lift:0:g"),
                        42.0
                    )
                ),
                FunctionDefinition::thunk(
                    "mir:lift:0:g",
                    Type::Number,
                    Let::new("g", function_type, Variable::new("mir:lift:0:g"), 42.0),
                )
            ])
        );
    }

    #[test]
    fn lift_thunk_with_free_variable() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::new(
            "f",
            vec![Argument::new("x", Type::Number)],
            Type::Number,
            LetRecursive::new(
                FunctionDefinition::thunk("g", Type::Number, Variable::new("x"))
                    .set_environment(vec![Argument::new("x", Type::Number)]),
                42.0,
            ),
        )]);

        assert_eq!(transform(&module), module);
    }
}
