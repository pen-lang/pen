mod alias_removal;
mod context;

use self::context::Context;
use crate::{ir::*, types::Type};
use std::convert::identity;

// Normalize expressions into the A-normal form with some exceptions.
//
// - Let and let-recursive expressions are flattened.
// - Arguments do not have to be variables.
// - Conditional expressions are kept nested.
//   - Otherwise, we need to duplicate continuations of those expression.
//
// This transformation assumes that the alpha conversion is applied to a module
// already.
pub fn transform(module: &Module) -> Module {
    let context = Context::new(module);

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
                    transform_function_definition(&context, definition.definition()),
                    definition.is_public(),
                )
            })
            .collect(),
    )
}

fn transform_function_definition(
    context: &Context,
    definition: &FunctionDefinition,
) -> FunctionDefinition {
    FunctionDefinition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        definition.result_type().clone(),
        alias_removal::transform(&transform_expression(context, definition.body(), &identity)),
        definition.is_thunk(),
    )
}

fn transform_expression(
    context: &Context,
    expression: &Expression,
    continue_: &dyn Fn(Expression) -> Expression,
) -> Expression {
    let transform_expression = |expression, continue_: &dyn Fn(Expression) -> Expression| {
        transform_expression(context, expression, continue_)
    };

    match expression {
        Expression::ArithmeticOperation(operation) => {
            transform_bound_expression(context, operation.lhs(), &Type::Number, &|lhs| {
                transform_expression(operation.rhs(), &|rhs| {
                    continue_(
                        ArithmeticOperation::new(operation.operator(), lhs.clone(), rhs).into(),
                    )
                })
            })
        }
        Expression::Call(call) => transform_bound_expression(
            context,
            call.function(),
            &call.type_().clone().into(),
            &|function| {
                transform_expressions(
                    context,
                    &call
                        .arguments()
                        .iter()
                        .zip(call.type_().arguments())
                        .collect::<Vec<_>>(),
                    &|arguments| {
                        continue_(
                            Call::new(call.type_().clone(), function.clone(), arguments).into(),
                        )
                    },
                )
            },
        ),
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
            transform_bound_expression(context, operation.lhs(), &Type::Number, &|lhs| {
                transform_expression(operation.rhs(), &|rhs| {
                    continue_(
                        ComparisonOperation::new(operation.operator(), lhs.clone(), rhs).into(),
                    )
                })
            })
        }
        Expression::DropVariables(drop) => transform_expression(drop.expression(), &|expression| {
            continue_(CloneVariables::new(drop.variables().clone(), expression).into())
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
            transform_function_definition(context, let_.definition()),
            transform_expression(let_.expression(), continue_),
        )
        .into(),
        Expression::Synchronize(synchronize) => {
            transform_expression(synchronize.expression(), &|expression| {
                continue_(Synchronize::new(synchronize.type_().clone(), expression).into())
            })
        }
        Expression::Record(record) => transform_expressions(
            context,
            &record
                .fields()
                .iter()
                .zip(context.record_fields()[record.type_().name()].fields())
                .collect::<Vec<_>>(),
            &|fields| continue_(Record::new(record.type_().clone(), fields).into()),
        ),
        Expression::RecordField(field) => transform_expression(field.record(), &|expression| {
            continue_(RecordField::new(field.type_().clone(), field.index(), expression).into())
        }),
        Expression::RecordUpdate(update) => transform_bound_expression(
            context,
            update.record(),
            &update.type_().clone().into(),
            &|record| {
                transform_expressions(
                    context,
                    &update
                        .fields()
                        .iter()
                        .map(|field| field.expression())
                        .zip(update.fields().iter().map(|field| {
                            &context.record_fields()[update.type_().name()].fields()[field.index()]
                        }))
                        .collect::<Vec<_>>(),
                    &|fields| {
                        continue_(
                            RecordUpdate::new(
                                update.type_().clone(),
                                record.clone(),
                                update
                                    .fields()
                                    .iter()
                                    .zip(fields)
                                    .map(|(field, expression)| {
                                        RecordUpdateField::new(field.index(), expression)
                                    })
                                    .collect(),
                            )
                            .into(),
                        )
                    },
                )
            },
        ),
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
    }
}

fn transform_bound_expression(
    context: &Context,
    bound_expression: &Expression,
    type_: &Type,
    continue_: &dyn Fn(Expression) -> Expression,
) -> Expression {
    transform_expression(
        context,
        bound_expression,
        &|bound_expression| match &bound_expression {
            Expression::Boolean(_)
            | Expression::ByteString(_)
            | Expression::None
            | Expression::Number(_)
            | Expression::Variable(_) => continue_(bound_expression),
            _ => {
                let name = context.generate_name();

                Let::new(
                    &name,
                    type_.clone(),
                    bound_expression,
                    continue_(Variable::new(&name).into()),
                )
                .into()
            }
        },
    )
}

fn transform_expressions(
    context: &Context,
    expressions: &[(&Expression, &Type)],
    continue_: &impl Fn(Vec<Expression>) -> Expression,
) -> Expression {
    transform_expressions_recursively(context, expressions, vec![], &continue_)
}

fn transform_expressions_recursively(
    context: &Context,
    expressions: &[(&Expression, &Type)],
    transformed_expressions: Vec<Expression>,
    continue_: &impl Fn(Vec<Expression>) -> Expression,
) -> Expression {
    match expressions {
        [] => continue_(transformed_expressions),
        [(expression, type_), ..] => {
            transform_bound_expression(context, expression, type_, &|expression| {
                transform_expressions_recursively(
                    context,
                    &expressions[1..],
                    // TODO This is O(n^2)!
                    transformed_expressions
                        .iter()
                        .cloned()
                        .chain([expression])
                        .collect(),
                    continue_,
                )
            })
        }
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
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::new(
            "f", vec![], Type::Number, 42.0)]);

        assert_eq!(transform(&module), module);
    }

    #[test]
    fn transform_let() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::new(
            "f", vec![], Type::Number, Let::new("x", Type::Number, 42.0, Variable::new("x")))]);

        assert_eq!(transform(&module), module);
    }

    #[test]
    fn transform_nested_let() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, Let::new(
                        "x",
                        Type::Number,
                        Let::new("y", Type::Number, 42.0, Variable::new("y")),
                        Variable::new("x"),
                    ))])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f", vec![], Type::Number, Let::new("y", Type::Number, 42.0, Variable::new("y")))])
        );
    }

    #[test]
    fn transform_deeply_nested_let() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, Let::new(
                        "x",
                        Type::Number,
                        Let::new(
                            "y",
                            Type::Number,
                            Let::new("z", Type::Number, 42.0, Variable::new("z")),
                            Variable::new("y")
                        ),
                        Variable::new("x"),
                    ))])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f", vec![], Type::Number, Let::new("z", Type::Number, 42.0, Variable::new("z")))])
        );
    }

    #[test]
    fn transform_nested_let_in_nested_expression() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, Variant::new(
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
                    ))])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f", vec![], Type::Number, Let::new(
                    "y",
                    Type::None,
                    Expression::None,
                    Let::new(
                        "x",
                        Type::None,
                        Variant::new(Type::None, Variable::new("y")),
                        Variant::new(Type::None, Variable::new("x")),
                    )
                ))])
        );
    }

    #[test]
    fn transform_arithmetic_operation() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Let::new("x", Type::Number, 1.0, Variable::new("x")),
                        Let::new("y", Type::Number, 2.0, Variable::new("y")),
                    ))])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f", vec![], Type::Number, Let::new(
                    "x",
                    Type::Number,
                    1.0,
                    Let::new(
                        "y",
                        Type::Number,
                        2.0,
                        ArithmeticOperation::new(
                            ArithmeticOperator::Add,
                            Variable::new("x"),
                            Variable::new("y"),
                        )
                    )
                ))])
        );
    }

    #[test]
    fn transform_normalized_call() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, Call::new(
                        types::Function::new(vec![Type::Number, Type::Number], Type::Number),
                        Let::new("x", Type::Number, 1.0, Variable::new("x")),
                        vec![
                            Let::new("y", Type::Number, 2.0, Variable::new("y")).into(),
                            Let::new("z", Type::Number, 3.0, Variable::new("z")).into(),
                        ],
                    ))])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f", vec![], Type::Number, Let::new(
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
                ))])
        );
    }

    #[test]
    fn transform_call() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, Call::new(
                        types::Function::new(vec![Type::Variant, Type::Variant], Type::Number),
                        Let::new(
                            "x",
                            Type::Number,
                            1.0,
                            Variant::new(Type::Number, Variable::new("x"))
                        ),
                        vec![
                            Let::new(
                                "y",
                                Type::Number,
                                2.0,
                                Variant::new(Type::Number, Variable::new("y"))
                            )
                            .into(),
                            Let::new(
                                "z",
                                Type::Number,
                                3.0,
                                Variant::new(Type::Number, Variable::new("z"))
                            )
                            .into(),
                        ],
                    ))])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f", vec![], Type::Number, Let::new(
                    "x",
                    Type::Number,
                    1.0,
                    Let::new(
                        "anf:v:0",
                        types::Function::new(vec![Type::Variant, Type::Variant], Type::Number),
                        Variant::new(Type::Number, Variable::new("x")),
                        Let::new(
                            "y",
                            Type::Number,
                            2.0,
                            Let::new(
                                "anf:v:1",
                                Type::Variant,
                                Variant::new(Type::Number, Variable::new("y")),
                                Let::new(
                                    "z",
                                    Type::Number,
                                    3.0,
                                    Let::new(
                                        "anf:v:2",
                                        Type::Variant,
                                        Variant::new(Type::Number, Variable::new("z")),
                                        Call::new(
                                            types::Function::new(
                                                vec![Type::Variant, Type::Variant],
                                                Type::Number
                                            ),
                                            Variable::new("anf:v:0"),
                                            vec![
                                                Variable::new("anf:v:1").into(),
                                                Variable::new("anf:v:2").into()
                                            ],
                                        ),
                                    )
                                )
                            )
                        )
                    )
                ))])
        );
    }

    #[test]
    fn transform_comparison_operation() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, ComparisonOperation::new(
                        ComparisonOperator::Equal,
                        Let::new("x", Type::Number, 1.0, Variable::new("x")),
                        Let::new("y", Type::Number, 2.0, Variable::new("y")),
                    ))])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f", vec![], Type::Number, Let::new(
                    "x",
                    Type::Number,
                    1.0,
                    Let::new(
                        "y",
                        Type::Number,
                        2.0,
                        ComparisonOperation::new(
                            ComparisonOperator::Equal,
                            Variable::new("x"),
                            Variable::new("y"),
                        ),
                    )
                ))])
        );
    }

    #[test]
    fn transform_record() {
        let type_definitions = vec![TypeDefinition::new(
            "r",
            types::RecordBody::new(vec![Type::Variant, Type::Variant]),
        )];

        assert_eq!(
            transform(
                &Module::empty()
                    .set_type_definitions(type_definitions.clone())
                    .set_function_definitions(vec![FunctionDefinition::new(
                        "f", vec![], Type::Number, Record::new(
                            types::Record::new("r"),
                            vec![
                                Let::new(
                                    "x",
                                    Type::Number,
                                    1.0,
                                    Variant::new(Type::Number, Variable::new("x"))
                                )
                                .into(),
                                Let::new(
                                    "y",
                                    Type::Number,
                                    2.0,
                                    Variant::new(Type::Number, Variable::new("y"))
                                )
                                .into(),
                            ],
                        ))])
            ),
            Module::empty()
                .set_type_definitions(type_definitions)
                .set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, Let::new(
                        "x",
                        Type::Number,
                        1.0,
                        Let::new(
                            "anf:v:0",
                            Type::Variant,
                            Variant::new(Type::Number, Variable::new("x")),
                            Let::new(
                                "y",
                                Type::Number,
                                2.0,
                                Let::new(
                                    "anf:v:1",
                                    Type::Variant,
                                    Variant::new(Type::Number, Variable::new("y")),
                                    Record::new(
                                        types::Record::new("r"),
                                        vec![
                                            Variable::new("anf:v:0").into(),
                                            Variable::new("anf:v:1").into(),
                                        ],
                                    )
                                )
                            )
                        )
                    ))])
        );
    }

    #[test]
    fn transform_record_field() {
        let type_definitions = vec![TypeDefinition::new(
            "r",
            types::RecordBody::new(vec![Type::Variant, Type::Variant]),
        )];

        assert_eq!(
            transform(
                &Module::empty()
                    .set_type_definitions(type_definitions.clone())
                    .set_function_definitions(vec![FunctionDefinition::new(
                        "f", vec![], Type::Number, RecordField::new(
                            types::Record::new("r"),
                            0,
                            Let::new(
                                "x",
                                Type::Number,
                                1.0,
                                Variant::new(Type::Number, Variable::new("x"))
                            )
                        ))])
            ),
            Module::empty()
                .set_type_definitions(type_definitions)
                .set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, Let::new(
                        "x",
                        Type::Number,
                        1.0,
                        RecordField::new(
                            types::Record::new("r"),
                            0,
                            Variant::new(Type::Number, Variable::new("x"))
                        )
                    ))])
        );
    }

    #[test]
    fn transform_record_update() {
        let type_definitions = vec![TypeDefinition::new(
            "r",
            types::RecordBody::new(vec![Type::Variant, Type::Variant]),
        )];

        assert_eq!(
            transform(
                &Module::empty()
                    .set_type_definitions(type_definitions.clone())
                    .set_function_definitions(vec![FunctionDefinition::new(
                        "f", vec![], Type::Number, RecordUpdate::new(
                            types::Record::new("r"),
                            Let::new(
                                "x",
                                Type::Number,
                                1.0,
                                Variant::new(Type::Number, Variable::new("x"))
                            ),
                            vec![
                                RecordUpdateField::new(
                                    0,
                                    Let::new(
                                        "y",
                                        Type::Number,
                                        2.0,
                                        Variant::new(Type::Number, Variable::new("y"))
                                    )
                                ),
                                RecordUpdateField::new(
                                    1,
                                    Let::new(
                                        "z",
                                        Type::Number,
                                        3.0,
                                        Variant::new(Type::Number, Variable::new("z"))
                                    )
                                ),
                            ],
                        ))])
            ),
            Module::empty()
                .set_type_definitions(type_definitions)
                .set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, Let::new(
                        "x",
                        Type::Number,
                        1.0,
                        Let::new(
                            "anf:v:0",
                            types::Record::new("r"),
                            Variant::new(Type::Number, Variable::new("x")),
                            Let::new(
                                "y",
                                Type::Number,
                                2.0,
                                Let::new(
                                    "anf:v:1",
                                    Type::Variant,
                                    Variant::new(Type::Number, Variable::new("y")),
                                    Let::new(
                                        "z",
                                        Type::Number,
                                        3.0,
                                        Let::new(
                                            "anf:v:2",
                                            Type::Variant,
                                            Variant::new(Type::Number, Variable::new("z")),
                                            RecordUpdate::new(
                                                types::Record::new("r"),
                                                Variable::new("anf:v:0"),
                                                vec![
                                                    RecordUpdateField::new(
                                                        0,
                                                        Variable::new("anf:v:1")
                                                    ),
                                                    RecordUpdateField::new(
                                                        1,
                                                        Variable::new("anf:v:2")
                                                    ),
                                                ],
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))])
        );
    }

    #[test]
    fn transform_try_operation() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, TryOperation::new(
                        Let::new("x", Type::None, Expression::None, Variable::new("x")),
                        "e",
                        Type::None,
                        Variable::new("e"),
                    ))])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f", vec![], Type::Number, Let::new(
                    "x",
                    Type::None,
                    Expression::None,
                    TryOperation::new(Variable::new("x"), "e", Type::None, Variable::new("e")),
                ))])
        );
    }

    #[test]
    fn transform_variant() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f", vec![], Type::Number, Variant::new(
                        Type::None,
                        Let::new("x", Type::None, Expression::None, Variable::new("x")),
                    ))])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f", vec![], Type::Number, Let::new(
                    "x",
                    Type::None,
                    Expression::None,
                    Variant::new(Type::None, Variable::new("x"))
                ))])
        );
    }
}
