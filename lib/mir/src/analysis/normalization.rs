use crate::ir::*;
use std::convert::identity;

// Normalize expressions into the A-normal form with some exceptions.
//
// - Let and let-recursive expressions are flattened.
// - Arguments do not have to be variables.
// - Conditional expressions are kept nested.
//   - Otherwise, we need to duplicate continuations of those expression.
pub fn transform(module: &Module) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(|definition| {
                GlobalFunctionDefinition::new(
                    transform_function_definition(definition.definition()),
                    definition.is_public(),
                )
            })
            .collect(),
    )
}

fn transform_function_definition(definition: &FunctionDefinition) -> FunctionDefinition {
    FunctionDefinition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        definition.result_type().clone(),
        transform_expression(definition.body(), &identity),
        definition.is_thunk(),
    )
}

fn transform_expression(
    expression: &Expression,
    continue_: &dyn Fn(Expression) -> Expression,
) -> Expression {
    match expression {
        Expression::ArithmeticOperation(operation) => {
            transform_expression(operation.lhs(), &|lhs| {
                transform_expression(operation.rhs(), &|rhs| {
                    continue_(
                        ArithmeticOperation::new(operation.operator(), lhs.clone(), rhs).into(),
                    )
                })
            })
        }
        Expression::Case(case) => transform_expression(case.argument(), &|argument| {
            continue_(
                Case::new(
                    argument,
                    case.alternatives()
                        .iter()
                        .map(|alternative| {
                            Alternative::new(
                                alternative.types().to_vec(),
                                alternative.name(),
                                transform_expression(alternative.expression(), &identity),
                            )
                        })
                        .collect(),
                    case.default_alternative().map(|alternative| {
                        DefaultAlternative::new(
                            alternative.name(),
                            transform_expression(alternative.expression(), &identity),
                        )
                    }),
                )
                .into(),
            )
        }),
        Expression::CloneVariables(clone) => {
            transform_expression(clone.expression(), &|expression| {
                continue_(CloneVariables::new(clone.variables().clone(), expression).into())
            })
        }
        Expression::ComparisonOperation(operation) => {
            transform_expression(operation.lhs(), &|lhs| {
                transform_expression(operation.rhs(), &|rhs| {
                    continue_(
                        ComparisonOperation::new(operation.operator(), lhs.clone(), rhs).into(),
                    )
                })
            })
            .into()
        }
        Expression::DropVariables(drop) => transform_expression(drop.expression(), &|expression| {
            continue_(CloneVariables::new(drop.variables().clone(), expression).into())
        }),
        Expression::Call(call) => transform_expression(call.function(), &|function| {
            transform_expressions(call.arguments(), &mut |arguments| {
                continue_(Call::new(call.type_().clone(), function.clone(), arguments).into())
            })
        }),
        Expression::If(if_) => transform_expression(if_.condition(), &|condition| {
            continue_(
                If::new(
                    condition,
                    transform_expression(if_.then(), &identity),
                    transform_expression(if_.else_(), &identity),
                )
                .into(),
            )
        }),
        Expression::Let(let_) => {
            transform_expression(let_.bound_expression(), &|bound_expression| {
                Let::new(
                    let_.name(),
                    let_.type_().clone(),
                    bound_expression,
                    transform_expression(let_.expression(), continue_),
                )
                .into()
            })
        }
        Expression::LetRecursive(let_) => LetRecursive::new(
            transform_function_definition(let_.definition()),
            transform_expression(let_.expression(), continue_),
        )
        .into(),
        Expression::Synchronize(synchronize) => {
            transform_expression(synchronize.expression(), &|expression| {
                continue_(Synchronize::new(synchronize.type_().clone(), expression).into())
            })
        }
        // Expression::Record(record) => Record::new(
        //     record.type_().clone(),
        //     record.fields().iter().map(transform_expression).collect(),
        // )
        // .into(),
        // Expression::RecordField(field) => RecordField::new(
        //     field.type_().clone(),
        //     field.index(),
        //     transform_expression(field.record()),
        // )
        // .into(),
        // Expression::RecordUpdate(update) => RecordUpdate::new(
        //     update.type_().clone(),
        //     transform_expression(update.record()),
        //     update
        //         .fields()
        //         .iter()
        //         .map(|field| {
        //             RecordUpdateField::new(field.index(), transform_expression(field.expression()))
        //         })
        //         .collect(),
        // )
        // .into(),
        Expression::TryOperation(operation) => {
            transform_expression(operation.operand(), &|operand| {
                continue_(
                    TryOperation::new(
                        operand,
                        operation.name(),
                        operation.type_().clone(),
                        transform_expression(operation.then(), &identity),
                    )
                    .into(),
                )
            })
        }
        Expression::Variant(variant) => transform_expression(variant.payload(), &|expression| {
            continue_(Variant::new(variant.type_().clone(), expression).into())
        }),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => continue_(expression.clone()),
        _ => todo!(),
    }
}

fn transform_expressions(
    expressions: &[Expression],
    continue_: &impl Fn(Vec<Expression>) -> Expression,
) -> Expression {
    transform_expressions_recursively(expressions, vec![], &continue_)
}

fn transform_expressions_recursively(
    expressions: &[Expression],
    transformed_expressions: Vec<Expression>,
    continue_: &impl Fn(Vec<Expression>) -> Expression,
) -> Expression {
    match expressions {
        [] => continue_(transformed_expressions),
        [expression, ..] => transform_expression(expression, &mut move |expression| {
            transform_expressions_recursively(
                &expressions[1..],
                transformed_expressions
                    .iter()
                    .cloned()
                    .chain([expression])
                    .collect(),
                continue_,
            )
        }),
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
    fn transform_function_definition() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            vec![],
            42.0,
            Type::Number,
        )]);

        assert_eq!(transform(&module), module);
    }

    #[test]
    fn transform_let() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            vec![],
            Let::new("x", Type::Number, 42.0, Variable::new("x")),
            Type::Number,
        )]);

        assert_eq!(transform(&module), module);
    }

    #[test]
    fn transform_nested_let() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "f",
                    vec![],
                    Let::new(
                        "x",
                        Type::Number,
                        Let::new("y", Type::Number, 42.0, Variable::new("y")),
                        Variable::new("x"),
                    ),
                    Type::Number,
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                vec![],
                Let::new(
                    "y",
                    Type::Number,
                    42.0,
                    Let::new("x", Type::Number, Variable::new("y"), Variable::new("x"))
                ),
                Type::Number,
            )])
        );
    }

    #[test]
    fn transform_deeply_nested_let() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "f",
                    vec![],
                    Let::new(
                        "x",
                        Type::Number,
                        Let::new(
                            "y",
                            Type::Number,
                            Let::new("z", Type::Number, 42.0, Variable::new("z")),
                            Variable::new("y")
                        ),
                        Variable::new("x"),
                    ),
                    Type::Number,
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                vec![],
                Let::new(
                    "z",
                    Type::Number,
                    42.0,
                    Let::new(
                        "y",
                        Type::Number,
                        Variable::new("z"),
                        Let::new("x", Type::Number, Variable::new("y"), Variable::new("x"))
                    )
                ),
                Type::Number,
            )])
        );
    }

    #[test]
    fn transform_nested_let_in_nested_expression() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "f",
                    vec![],
                    Variant::new(
                        Type::None,
                        Let::new(
                            "x",
                            Type::None,
                            Variant::new(
                                Type::None,
                                Let::new("y", Type::None, Expression::None, Variable::new("y"))
                            ),
                            Variable::new("x"),
                        )
                    ),
                    Type::Number,
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                vec![],
                Let::new(
                    "y",
                    Type::None,
                    Expression::None,
                    Let::new(
                        "x",
                        Type::None,
                        Variant::new(Type::None, Variable::new("y")),
                        Variant::new(Type::None, Variable::new("x")),
                    )
                ),
                Type::Number,
            )])
        );
    }

    #[test]
    fn transform_call() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "f",
                    vec![],
                    Call::new(
                        types::Function::new(vec![Type::Number, Type::Number], Type::Number),
                        Let::new("x", Type::Number, 1.0, Variable::new("x")),
                        vec![
                            Let::new("y", Type::Number, 2.0, Variable::new("y")).into(),
                            Let::new("z", Type::Number, 3.0, Variable::new("z")).into(),
                        ],
                    ),
                    Type::Number,
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                vec![],
                Let::new(
                    "x",
                    Type::Number,
                    1.0,
                    Let::new(
                        "y",
                        Type::Number,
                        2.0,
                        Let::new(
                            "z",
                            Type::Number,
                            3.0,
                            Call::new(
                                types::Function::new(
                                    vec![Type::Number, Type::Number],
                                    Type::Number
                                ),
                                Variable::new("x"),
                                vec![Variable::new("y").into(), Variable::new("z").into()],
                            ),
                        )
                    )
                ),
                Type::Number,
            )])
        );
    }

    #[test]
    fn transform_try_operation() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "f",
                    vec![],
                    TryOperation::new(
                        Let::new("x", Type::None, Expression::None, Variable::new("x")),
                        "e",
                        Type::None,
                        Variable::new("e"),
                    ),
                    Type::Number,
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                vec![],
                Let::new(
                    "x",
                    Type::None,
                    Expression::None,
                    TryOperation::new(Variable::new("x"), "e", Type::None, Variable::new("e")),
                ),
                Type::Number,
            )])
        );
    }

    #[test]
    fn transform_variant() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "f",
                    vec![],
                    Variant::new(
                        Type::None,
                        Let::new("x", Type::None, Expression::None, Variable::new("x")),
                    ),
                    Type::Number,
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                vec![],
                Let::new(
                    "x",
                    Type::None,
                    Expression::None,
                    Variant::new(Type::None, Variable::new("x"))
                ),
                Type::Number,
            )])
        );
    }
}
