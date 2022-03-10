use super::{
    context::CompileContext,
    transformation::{
        boolean_operation_transformer, equal_operation_transformer, if_list_transformer,
        not_equal_operation_transformer,
    },
    type_compiler, CompileError,
};
use crate::{
    concurrency_configuration::MODULE_LOCAL_SPAWN_FUNCTION_NAME,
    downcast_compiler,
    transformation::{
        list_literal_transformer, map_literal_transformer, record_update_transformer,
    },
};
use fnv::FnvHashMap;
use hir::{
    analysis::{
        record_field_resolver, type_canonicalizer, type_equality_checker,
        union_type_member_calculator, AnalysisError,
    },
    ir::*,
    types::{self, Type},
};

pub fn compile(
    expression: &Expression,
    context: &CompileContext,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(expression, context);

    Ok(match expression {
        Expression::Boolean(boolean) => mir::ir::Expression::Boolean(boolean.value()),
        Expression::Call(call) => mir::ir::Call::new(
            type_compiler::compile(
                call.function_type()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(call.position().clone()))?,
                context,
            )?
            .into_function()
            .ok_or_else(|| AnalysisError::FunctionExpected(call.position().clone()))?,
            compile(call.function())?,
            call.arguments()
                .iter()
                .map(compile)
                .collect::<Result<_, _>>()?,
        )
        .into(),
        Expression::If(if_) => mir::ir::If::new(
            compile(if_.condition())?,
            compile(if_.then())?,
            compile(if_.else_())?,
        )
        .into(),
        Expression::IfList(if_) => compile(&if_list_transformer::transform(if_, context)?)?,
        Expression::IfType(if_) => mir::ir::Case::new(
            compile(if_.argument())?,
            if_.branches()
                .iter()
                .map(|branch| {
                    compile_alternatives(if_.name(), branch.type_(), branch.expression(), context)
                })
                .collect::<Result<Vec<_>, CompileError>>()?
                .into_iter()
                .flatten()
                .chain(if let Some(branch) = if_.else_() {
                    if !type_equality_checker::check(
                        branch.type_().unwrap(),
                        &types::Any::new(if_.position().clone()).into(),
                        context.types(),
                    )? {
                        compile_alternatives(
                            if_.name(),
                            branch.type_().unwrap(),
                            branch.expression(),
                            context,
                        )?
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                })
                .collect(),
            if let Some(branch) = if_.else_() {
                if type_equality_checker::check(
                    branch.type_().unwrap(),
                    &types::Any::new(if_.position().clone()).into(),
                    context.types(),
                )? {
                    Some(mir::ir::DefaultAlternative::new(
                        if_.name(),
                        compile(branch.expression())?,
                    ))
                } else {
                    None
                }
            } else {
                None
            },
        )
        .into(),
        Expression::Lambda(lambda) => compile_lambda(lambda, context)?,
        Expression::Let(let_) => mir::ir::Let::new(
            let_.name().unwrap_or_default(),
            type_compiler::compile(
                let_.type_()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(let_.position().clone()))?,
                context,
            )?,
            compile(let_.bound_expression())?,
            compile(let_.expression())?,
        )
        .into(),
        Expression::List(list) => compile(&list_literal_transformer::transform(
            list,
            &context.configuration()?.list_type,
        ))?,
        Expression::ListComprehension(comprehension) => {
            const CLOSURE_NAME: &str = "$loop";
            const LIST_NAME: &str = "$list";

            let position = comprehension.position();
            let input_element_type = comprehension
                .input_type()
                .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;
            let output_element_type = comprehension.output_type();
            let list_type = type_compiler::compile_list(context)?;

            mir::ir::Call::new(
                mir::types::Function::new(
                    vec![mir::types::Function::new(vec![], list_type.clone()).into()],
                    list_type.clone(),
                ),
                mir::ir::Variable::new(&context.configuration()?.list_type.lazy_function_name),
                vec![mir::ir::LetRecursive::new(
                    mir::ir::Definition::new(
                        CLOSURE_NAME,
                        vec![],
                        mir::ir::LetRecursive::new(
                            mir::ir::Definition::new(
                                CLOSURE_NAME,
                                vec![mir::ir::Argument::new(LIST_NAME, list_type.clone())],
                                compile(
                                    &IfList::new(
                                        Some(input_element_type.clone()),
                                        Variable::new(LIST_NAME, position.clone()),
                                        comprehension.element_name(),
                                        LIST_NAME,
                                        List::new(
                                            output_element_type.clone(),
                                            vec![
                                                ListElement::Single(
                                                    comprehension.element().clone(),
                                                ),
                                                ListElement::Multiple(
                                                    Call::new(
                                                        Some(
                                                            types::Function::new(
                                                                vec![types::List::new(
                                                                    input_element_type.clone(),
                                                                    position.clone(),
                                                                )
                                                                .into()],
                                                                types::List::new(
                                                                    output_element_type.clone(),
                                                                    position.clone(),
                                                                ),
                                                                position.clone(),
                                                            )
                                                            .into(),
                                                        ),
                                                        Variable::new(
                                                            CLOSURE_NAME,
                                                            position.clone(),
                                                        ),
                                                        vec![Variable::new(
                                                            LIST_NAME,
                                                            position.clone(),
                                                        )
                                                        .into()],
                                                        position.clone(),
                                                    )
                                                    .into(),
                                                ),
                                            ],
                                            position.clone(),
                                        ),
                                        List::new(
                                            output_element_type.clone(),
                                            vec![],
                                            position.clone(),
                                        ),
                                        position.clone(),
                                    )
                                    .into(),
                                )?,
                                list_type.clone(),
                            ),
                            mir::ir::Call::new(
                                mir::types::Function::new(
                                    vec![list_type.clone().into()],
                                    list_type.clone(),
                                ),
                                mir::ir::Variable::new(CLOSURE_NAME),
                                vec![compile(comprehension.list())?],
                            ),
                        ),
                        list_type,
                    ),
                    mir::ir::Variable::new(CLOSURE_NAME),
                )
                .into()],
            )
            .into()
        }
        Expression::Map(map) => compile(&map_literal_transformer::transform(context, map)?)?,
        Expression::None(_) => mir::ir::Expression::None,
        Expression::Number(number) => mir::ir::Expression::Number(number.value()),
        Expression::Operation(operation) => compile_operation(operation, context)?,
        Expression::RecordConstruction(construction) => {
            let field_types = record_field_resolver::resolve(
                construction.type_(),
                construction.position(),
                context.types(),
                context.records(),
            )?;
            let record_type = type_compiler::compile(construction.type_(), context)?
                .into_record()
                .unwrap();

            compile_record_fields(
                construction.fields(),
                field_types,
                &|fields| {
                    mir::ir::Record::new(
                        record_type.clone(),
                        field_types
                            .iter()
                            .map(|field_type| fields[field_type.name()].clone())
                            .collect(),
                    )
                    .into()
                },
                context,
            )?
        }
        Expression::RecordDeconstruction(deconstruction) => {
            let type_ = deconstruction.type_().unwrap();

            mir::ir::RecordField::new(
                type_compiler::compile(type_, context)?
                    .into_record()
                    .unwrap(),
                record_field_resolver::resolve(
                    type_,
                    deconstruction.position(),
                    context.types(),
                    context.records(),
                )?
                .iter()
                .position(|field_type| field_type.name() == deconstruction.field_name())
                .unwrap(),
                compile(deconstruction.record())?,
            )
            .into()
        }
        Expression::RecordUpdate(update) => {
            compile(&record_update_transformer::transform(update, context)?)?
        }
        Expression::String(string) => mir::ir::ByteString::new(string.value()).into(),
        Expression::Thunk(thunk) => {
            const THUNK_NAME: &str = "$thunk";

            mir::ir::LetRecursive::new(
                mir::ir::Definition::thunk(
                    THUNK_NAME,
                    compile(thunk.expression())?,
                    type_compiler::compile(
                        thunk.type_().ok_or_else(|| {
                            AnalysisError::TypeNotInferred(thunk.position().clone())
                        })?,
                        context,
                    )?,
                ),
                mir::ir::Variable::new(THUNK_NAME),
            )
            .into()
        }
        Expression::TypeCoercion(coercion) => {
            let from = type_canonicalizer::canonicalize(coercion.from(), context.types())?;
            let to = type_canonicalizer::canonicalize(coercion.to(), context.types())?;
            let argument = compile(coercion.argument())?;

            if from.is_list() && to.is_list() {
                argument
            } else {
                match &from {
                    Type::Boolean(_)
                    | Type::None(_)
                    | Type::Number(_)
                    | Type::Record(_)
                    | Type::String(_) => mir::ir::Variant::new(
                        type_compiler::compile(coercion.from(), context)?,
                        argument,
                    )
                    .into(),
                    Type::Function(function_type) => {
                        let concrete_type = type_compiler::compile_concrete_function(
                            function_type,
                            context.types(),
                        )?;

                        mir::ir::Variant::new(
                            concrete_type.clone(),
                            mir::ir::Record::new(concrete_type, vec![argument]),
                        )
                        .into()
                    }
                    Type::List(list_type) => {
                        let concrete_type =
                            type_compiler::compile_concrete_list(list_type, context.types())?;

                        mir::ir::Variant::new(
                            concrete_type.clone(),
                            mir::ir::Record::new(concrete_type, vec![argument]),
                        )
                        .into()
                    }
                    Type::Map(map_type) => {
                        let concrete_type =
                            type_compiler::compile_concrete_map(map_type, context.types())?;

                        mir::ir::Variant::new(
                            concrete_type.clone(),
                            mir::ir::Record::new(concrete_type, vec![argument]),
                        )
                        .into()
                    }
                    Type::Any(_) | Type::Union(_) => argument,
                    Type::Reference(_) => unreachable!(),
                }
            }
        }
        Expression::Variable(variable) => mir::ir::Variable::new(variable.name()).into(),
    })
}

fn compile_lambda(
    lambda: &hir::ir::Lambda,
    context: &CompileContext,
) -> Result<mir::ir::Expression, CompileError> {
    const CLOSURE_NAME: &str = "$closure";

    Ok(mir::ir::LetRecursive::new(
        mir::ir::Definition::new(
            CLOSURE_NAME,
            lambda
                .arguments()
                .iter()
                .map(|argument| -> Result<_, CompileError> {
                    Ok(mir::ir::Argument::new(
                        argument.name(),
                        type_compiler::compile(argument.type_(), context)?,
                    ))
                })
                .collect::<Result<_, _>>()?,
            compile(lambda.body(), context)?,
            type_compiler::compile(lambda.result_type(), context)?,
        ),
        mir::ir::Variable::new(CLOSURE_NAME),
    )
    .into())
}

fn compile_alternatives(
    name: &str,
    type_: &Type,
    expression: &Expression,
    context: &CompileContext,
) -> Result<Vec<mir::ir::Alternative>, CompileError> {
    let type_ = type_canonicalizer::canonicalize(type_, context.types())?;
    let expression = compile(expression, context)?;

    union_type_member_calculator::calculate(&type_, context.types())?
        .into_iter()
        .map(|member_type| {
            let compiled_member_type = type_compiler::compile(&member_type, context)?;

            Ok(match &member_type {
                Type::Function(function_type) => compile_generic_type_alternative(
                    name,
                    &expression,
                    &type_,
                    &compiled_member_type,
                    &type_compiler::compile_concrete_function(function_type, context.types())?,
                ),
                Type::List(list_type) => compile_generic_type_alternative(
                    name,
                    &expression,
                    &type_,
                    &compiled_member_type,
                    &type_compiler::compile_concrete_list(list_type, context.types())?,
                ),
                _ => mir::ir::Alternative::new(compiled_member_type.clone(), name, {
                    if type_.is_union() {
                        mir::ir::Let::new(
                            name,
                            mir::types::Type::Variant,
                            mir::ir::Variant::new(
                                compiled_member_type,
                                mir::ir::Variable::new(name),
                            ),
                            expression.clone(),
                        )
                        .into()
                    } else {
                        expression.clone()
                    }
                }),
            })
        })
        .collect::<Result<_, _>>()
}

fn compile_generic_type_alternative(
    name: &str,
    expression: &mir::ir::Expression,
    type_: &hir::types::Type,
    member_type: &mir::types::Type,
    concrete_member_type: &mir::types::Record,
) -> mir::ir::Alternative {
    mir::ir::Alternative::new(concrete_member_type.clone(), name, {
        if type_.is_union() {
            mir::ir::Let::new(
                name,
                mir::types::Type::Variant,
                mir::ir::Variant::new(concrete_member_type.clone(), mir::ir::Variable::new(name)),
                expression.clone(),
            )
        } else {
            mir::ir::Let::new(
                name,
                member_type.clone(),
                mir::ir::RecordField::new(
                    concrete_member_type.clone(),
                    0,
                    mir::ir::Variable::new(name),
                ),
                expression.clone(),
            )
        }
    })
}

fn compile_operation(
    operation: &Operation,
    context: &CompileContext,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(expression, context);

    Ok(match operation {
        Operation::Arithmetic(operation) => mir::ir::ArithmeticOperation::new(
            match operation.operator() {
                ArithmeticOperator::Add => mir::ir::ArithmeticOperator::Add,
                ArithmeticOperator::Subtract => mir::ir::ArithmeticOperator::Subtract,
                ArithmeticOperator::Multiply => mir::ir::ArithmeticOperator::Multiply,
                ArithmeticOperator::Divide => mir::ir::ArithmeticOperator::Divide,
            },
            compile(operation.lhs())?,
            compile(operation.rhs())?,
        )
        .into(),
        Operation::Spawn(operation) => compile_spawn_operation(operation, context)?,
        Operation::Boolean(operation) => {
            compile(&boolean_operation_transformer::transform(operation))?
        }
        Operation::Equality(operation) => match operation.operator() {
            EqualityOperator::Equal => {
                match type_canonicalizer::canonicalize(
                    operation.type_().ok_or_else(|| {
                        AnalysisError::TypeNotInferred(operation.position().clone())
                    })?,
                    context.types(),
                )? {
                    Type::Number(_) => mir::ir::ComparisonOperation::new(
                        mir::ir::ComparisonOperator::Equal,
                        compile(operation.lhs())?,
                        compile(operation.rhs())?,
                    )
                    .into(),
                    Type::String(_) => mir::ir::Call::new(
                        mir::types::Function::new(
                            vec![mir::types::Type::ByteString, mir::types::Type::ByteString],
                            mir::types::Type::Boolean,
                        ),
                        mir::ir::Variable::new(
                            &context.configuration()?.string_type.equal_function_name,
                        ),
                        vec![compile(operation.lhs())?, compile(operation.rhs())?],
                    )
                    .into(),
                    _ => compile(&equal_operation_transformer::transform(operation, context)?)?,
                }
            }
            EqualityOperator::NotEqual => {
                compile(&not_equal_operation_transformer::transform(operation))?
            }
        },
        Operation::Not(operation) => {
            mir::ir::If::new(compile(operation.expression())?, false, true).into()
        }
        Operation::Order(operation) => mir::ir::ComparisonOperation::new(
            match operation.operator() {
                OrderOperator::LessThan => mir::ir::ComparisonOperator::LessThan,
                OrderOperator::LessThanOrEqual => mir::ir::ComparisonOperator::LessThanOrEqual,
                OrderOperator::GreaterThan => mir::ir::ComparisonOperator::GreaterThan,
                OrderOperator::GreaterThanOrEqual => {
                    mir::ir::ComparisonOperator::GreaterThanOrEqual
                }
            },
            compile(operation.lhs())?,
            compile(operation.rhs())?,
        )
        .into(),
        Operation::Try(operation) => {
            let success_type = operation
                .type_()
                .ok_or_else(|| AnalysisError::TypeNotInferred(operation.position().clone()))?;
            let error_type = type_compiler::compile(
                &types::Reference::new(
                    &context.configuration()?.error_type.error_type_name,
                    operation.position().clone(),
                )
                .into(),
                context,
            )?;

            mir::ir::Case::new(
                mir::ir::TryOperation::new(
                    compile(operation.expression())?,
                    "$error",
                    error_type.clone(),
                    mir::ir::Variant::new(error_type, mir::ir::Variable::new("$error")),
                ),
                compile_alternatives(
                    "$success",
                    success_type,
                    &Variable::new("$success", operation.position().clone()).into(),
                    context,
                )?,
                None,
            )
            .into()
        }
    })
}

fn compile_spawn_operation(
    operation: &SpawnOperation,
    context: &CompileContext,
) -> Result<mir::ir::Expression, CompileError> {
    const ANY_THUNK_NAME: &str = "$any_thunk";
    const THUNK_NAME: &str = "$thunk";

    let position = operation.position();
    let body = operation.function().body();
    let result_type = operation.function().result_type();
    let any_type = Type::from(types::Any::new(position.clone()));
    let thunk_type = types::Function::new(vec![], any_type.clone(), position.clone()).into();

    Ok(mir::ir::Let::new(
        ANY_THUNK_NAME,
        type_compiler::compile(&thunk_type, context)?,
        mir::ir::Call::new(
            type_compiler::compile_spawn_function(),
            mir::ir::Variable::new(MODULE_LOCAL_SPAWN_FUNCTION_NAME),
            vec![mir::ir::LetRecursive::new(
                mir::ir::Definition::thunk(
                    ANY_THUNK_NAME,
                    compile(
                        &TypeCoercion::new(
                            result_type.clone(),
                            any_type.clone(),
                            body.clone(),
                            body.position().clone(),
                        )
                        .into(),
                        context,
                    )?,
                    type_compiler::compile(&any_type, context)?,
                ),
                mir::ir::Variable::new(ANY_THUNK_NAME),
            )
            .into()],
        ),
        mir::ir::LetRecursive::new(
            mir::ir::Definition::new(
                THUNK_NAME,
                vec![],
                compile(
                    &downcast_compiler::compile(
                        &any_type,
                        result_type,
                        &Call::new(
                            Some(thunk_type.clone()),
                            Variable::new(ANY_THUNK_NAME, position.clone()),
                            vec![],
                            position.clone(),
                        )
                        .into(),
                        context,
                    )?,
                    context,
                )?,
                type_compiler::compile(result_type, context)?,
            ),
            mir::ir::Variable::new(THUNK_NAME),
        ),
    )
    .into())
}

fn compile_record_fields(
    fields: &[RecordField],
    field_types: &[types::RecordField],
    convert_fields_to_expression: &dyn Fn(
        &FnvHashMap<String, mir::ir::Expression>,
    ) -> mir::ir::Expression,
    context: &CompileContext,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(match fields {
        [] => convert_fields_to_expression(&Default::default()),
        [field, ..] => {
            let field_name = format!("${}", field.name());

            mir::ir::Let::new(
                field_name.clone(),
                type_compiler::compile(
                    field_types
                        .iter()
                        .find(|field_type| field_type.name() == field.name())
                        .ok_or_else(|| AnalysisError::RecordFieldUnknown(field.position().clone()))?
                        .type_(),
                    context,
                )?,
                compile(field.expression(), context)?,
                compile_record_fields(
                    &fields[1..],
                    field_types,
                    &|fields| {
                        convert_fields_to_expression(
                            &fields
                                .clone()
                                .into_iter()
                                .chain([(
                                    field.name().into(),
                                    mir::ir::Variable::new(field_name.clone()).into(),
                                )])
                                .collect(),
                        )
                    },
                    context,
                )?,
            )
            .into()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::{test::PositionFake, Position};

    fn compile_expression(expression: &Expression) -> Result<mir::ir::Expression, CompileError> {
        compile(
            expression,
            &CompileContext::dummy(Default::default(), Default::default()),
        )
    }

    mod if_type {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_with_union() {
            assert_eq!(
                compile_expression(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![
                            IfTypeBranch::new(
                                types::Number::new(Position::fake()),
                                None::new(Position::fake()),
                            ),
                            IfTypeBranch::new(
                                types::None::new(Position::fake()),
                                None::new(Position::fake()),
                            )
                        ],
                        None,
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![
                        mir::ir::Alternative::new(
                            mir::types::Type::Number,
                            "y",
                            mir::ir::Expression::None
                        ),
                        mir::ir::Alternative::new(
                            mir::types::Type::None,
                            "y",
                            mir::ir::Expression::None
                        )
                    ],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_with_union_and_else() {
            assert_eq!(
                compile_expression(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Number::new(Position::fake()),
                            None::new(Position::fake()),
                        )],
                        Some(ElseBranch::new(
                            Some(types::None::new(Position::fake()).into()),
                            None::new(Position::fake()),
                            Position::fake()
                        )),
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![
                        mir::ir::Alternative::new(
                            mir::types::Type::Number,
                            "y",
                            mir::ir::Expression::None
                        ),
                        mir::ir::Alternative::new(
                            mir::types::Type::None,
                            "y",
                            mir::ir::Expression::None
                        )
                    ],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_with_any() {
            assert_eq!(
                compile_expression(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Number::new(Position::fake()),
                            None::new(Position::fake()),
                        )],
                        Some(ElseBranch::new(
                            Some(types::Any::new(Position::fake()).into()),
                            None::new(Position::fake()),
                            Position::fake()
                        )),
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        mir::types::Type::Number,
                        "y",
                        mir::ir::Expression::None
                    )],
                    Some(mir::ir::DefaultAlternative::new(
                        "y",
                        mir::ir::Expression::None
                    ))
                )
                .into())
            );
        }

        #[test]
        fn compile_with_union_branch() {
            assert_eq!(
                compile_expression(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Union::new(
                                types::Number::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake()
                            ),
                            None::new(Position::fake()),
                        )],
                        None,
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![
                        mir::ir::Alternative::new(
                            mir::types::Type::None,
                            "y",
                            mir::ir::Let::new(
                                "y",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::None,
                                    mir::ir::Variable::new("y")
                                ),
                                mir::ir::Expression::None
                            )
                        ),
                        mir::ir::Alternative::new(
                            mir::types::Type::Number,
                            "y",
                            mir::ir::Let::new(
                                "y",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::Number,
                                    mir::ir::Variable::new("y")
                                ),
                                mir::ir::Expression::None
                            )
                        ),
                    ],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_function_branch() {
            let context = CompileContext::dummy(Default::default(), Default::default());
            let function_type =
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());
            let concrete_function_type =
                type_compiler::compile_concrete_function(&function_type, context.types()).unwrap();

            assert_eq!(
                compile(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            function_type.clone(),
                            Variable::new("y", Position::fake()),
                        )],
                        None,
                        Position::fake(),
                    )
                    .into(),
                    &context,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        concrete_function_type.clone(),
                        "y",
                        mir::ir::Let::new(
                            "y",
                            type_compiler::compile_function(&function_type, &context).unwrap(),
                            mir::ir::RecordField::new(
                                concrete_function_type,
                                0,
                                mir::ir::Variable::new("y")
                            ),
                            mir::ir::Variable::new("y")
                        ),
                    )],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_list_branch() {
            let context = CompileContext::dummy(Default::default(), Default::default());
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
            let concrete_list_type =
                type_compiler::compile_concrete_list(&list_type, context.types()).unwrap();

            assert_eq!(
                compile(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            list_type,
                            Variable::new("y", Position::fake()),
                        )],
                        None,
                        Position::fake(),
                    )
                    .into(),
                    &context,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        concrete_list_type.clone(),
                        "y",
                        mir::ir::Let::new(
                            "y",
                            mir::types::Record::new(
                                &context.configuration().unwrap().list_type.list_type_name
                            ),
                            mir::ir::RecordField::new(
                                concrete_list_type,
                                0,
                                mir::ir::Variable::new("y")
                            ),
                            mir::ir::Variable::new("y")
                        ),
                    )],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_union_branch_including_list() {
            let context = CompileContext::dummy(Default::default(), Default::default());
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
            let concrete_list_type =
                type_compiler::compile_concrete_list(&list_type, context.types()).unwrap();

            assert_eq!(
                compile(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Union::new(
                                list_type,
                                types::None::new(Position::fake()),
                                Position::fake()
                            ),
                            Variable::new("y", Position::fake()),
                        )],
                        None,
                        Position::fake(),
                    )
                    .into(),
                    &context,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![
                        mir::ir::Alternative::new(
                            concrete_list_type.clone(),
                            "y",
                            mir::ir::Let::new(
                                "y",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    concrete_list_type,
                                    mir::ir::Variable::new("y")
                                ),
                                mir::ir::Variable::new("y")
                            ),
                        ),
                        mir::ir::Alternative::new(
                            mir::types::Type::None,
                            "y",
                            mir::ir::Let::new(
                                "y",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::None,
                                    mir::ir::Variable::new("y")
                                ),
                                mir::ir::Variable::new("y")
                            ),
                        ),
                    ],
                    None
                )
                .into())
            );
        }
    }

    mod records {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_record_construction() {
            assert_eq!(
                compile(
                    &RecordConstruction::new(
                        types::Record::new("r", Position::fake()),
                        vec![RecordField::new(
                            "x",
                            None::new(Position::fake()),
                            Position::fake()
                        )],
                        Position::fake()
                    )
                    .into(),
                    &CompileContext::dummy(
                        Default::default(),
                        [(
                            "r".into(),
                            vec![types::RecordField::new(
                                "x",
                                types::None::new(Position::fake())
                            )]
                        )]
                        .into_iter()
                        .collect()
                    ),
                ),
                Ok(mir::ir::Let::new(
                    "$x",
                    mir::types::Type::None,
                    mir::ir::Expression::None,
                    mir::ir::Record::new(
                        mir::types::Record::new("r"),
                        vec![mir::ir::Variable::new("$x").into()]
                    )
                )
                .into())
            );
        }

        #[test]
        fn compile_record_construction_with_two_fields() {
            assert_eq!(
                compile(
                    &RecordConstruction::new(
                        types::Record::new("r", Position::fake()),
                        vec![
                            RecordField::new(
                                "x",
                                Number::new(42.0, Position::fake()),
                                Position::fake()
                            ),
                            RecordField::new("y", None::new(Position::fake()), Position::fake())
                        ],
                        Position::fake()
                    )
                    .into(),
                    &CompileContext::dummy(
                        Default::default(),
                        [(
                            "r".into(),
                            vec![
                                types::RecordField::new("x", types::Number::new(Position::fake())),
                                types::RecordField::new("y", types::None::new(Position::fake()))
                            ]
                        )]
                        .into_iter()
                        .collect()
                    ),
                ),
                Ok(mir::ir::Let::new(
                    "$x",
                    mir::types::Type::Number,
                    42.0,
                    mir::ir::Let::new(
                        "$y",
                        mir::types::Type::None,
                        mir::ir::Expression::None,
                        mir::ir::Record::new(
                            mir::types::Record::new("r"),
                            vec![
                                mir::ir::Variable::new("$x").into(),
                                mir::ir::Variable::new("$y").into()
                            ]
                        )
                    )
                )
                .into())
            );
        }

        #[test]
        fn compile_singleton_record_construction() {
            assert_eq!(
                compile(
                    &RecordConstruction::new(
                        types::Record::new("r", Position::fake()),
                        vec![],
                        Position::fake()
                    )
                    .into(),
                    &CompileContext::dummy(
                        Default::default(),
                        [("r".into(), vec![])].into_iter().collect()
                    ),
                ),
                Ok(mir::ir::Record::new(mir::types::Record::new("r"), vec![]).into())
            );
        }

        #[test]
        fn compile_record_construction_with_reference_type() {
            assert_eq!(
                compile(
                    &RecordConstruction::new(
                        types::Reference::new("r", Position::fake()),
                        vec![],
                        Position::fake()
                    )
                    .into(),
                    &CompileContext::dummy(
                        [("r".into(), types::Record::new("r", Position::fake()).into())]
                            .into_iter()
                            .collect(),
                        [("r".into(), vec![])].into_iter().collect()
                    ),
                ),
                Ok(mir::ir::Record::new(mir::types::Record::new("r"), vec![]).into())
            );
        }
    }

    mod try_operation {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_with_none() {
            let error_type = mir::types::Record::new("error");

            assert_eq!(
                compile(
                    &TryOperation::new(
                        Some(types::None::new(Position::fake()).into()),
                        Variable::new("x", Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                    &CompileContext::dummy(
                        [(
                            "error".into(),
                            types::Record::new("error", Position::fake()).into()
                        )]
                        .into_iter()
                        .collect(),
                        Default::default()
                    ),
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::TryOperation::new(
                        mir::ir::Variable::new("x"),
                        "$error",
                        error_type.clone(),
                        mir::ir::Variant::new(error_type, mir::ir::Variable::new("$error"))
                    ),
                    vec![mir::ir::Alternative::new(
                        mir::types::Type::None,
                        "$success",
                        mir::ir::Variable::new("$success"),
                    )],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_with_union() {
            let error_type = mir::types::Record::new("error");

            assert_eq!(
                compile(
                    &TryOperation::new(
                        Some(
                            types::Union::new(
                                types::Number::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake()
                            )
                            .into()
                        ),
                        Variable::new("x", Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                    &CompileContext::dummy(
                        [(
                            "error".into(),
                            types::Record::new("error", Position::fake()).into()
                        )]
                        .into_iter()
                        .collect(),
                        Default::default()
                    ),
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::TryOperation::new(
                        mir::ir::Variable::new("x"),
                        "$error",
                        error_type.clone(),
                        mir::ir::Variant::new(error_type, mir::ir::Variable::new("$error"))
                    ),
                    vec![
                        mir::ir::Alternative::new(
                            mir::types::Type::None,
                            "$success",
                            mir::ir::Let::new(
                                "$success",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::None,
                                    mir::ir::Variable::new("$success")
                                ),
                                mir::ir::Variable::new("$success"),
                            ),
                        ),
                        mir::ir::Alternative::new(
                            mir::types::Type::Number,
                            "$success",
                            mir::ir::Let::new(
                                "$success",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::Number,
                                    mir::ir::Variable::new("$success")
                                ),
                                mir::ir::Variable::new("$success"),
                            ),
                        ),
                    ],
                    None
                )
                .into())
            );
        }
    }

    mod spawn_operation {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_spawn_operation() {
            assert_eq!(
                compile_expression(
                    &SpawnOperation::new(
                        Lambda::new(
                            vec![],
                            types::Number::new(Position::fake()),
                            Number::new(42.0, Position::fake()),
                            Position::fake()
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Let::new(
                    "$any_thunk",
                    mir::types::Function::new(vec![], mir::types::Type::Variant),
                    mir::ir::Call::new(
                        type_compiler::compile_spawn_function(),
                        mir::ir::Variable::new(MODULE_LOCAL_SPAWN_FUNCTION_NAME),
                        vec![mir::ir::LetRecursive::new(
                            mir::ir::Definition::thunk(
                                "$any_thunk",
                                mir::ir::Variant::new(
                                    mir::types::Type::Number,
                                    mir::ir::Expression::Number(42.0)
                                ),
                                mir::types::Type::Variant
                            ),
                            mir::ir::Variable::new("$any_thunk"),
                        )
                        .into()]
                    ),
                    mir::ir::LetRecursive::new(
                        mir::ir::Definition::new(
                            "$thunk",
                            vec![],
                            mir::ir::Case::new(
                                mir::ir::Call::new(
                                    mir::types::Function::new(vec![], mir::types::Type::Variant),
                                    mir::ir::Variable::new("$any_thunk"),
                                    vec![]
                                ),
                                vec![mir::ir::Alternative::new(
                                    mir::types::Type::Number,
                                    "$value",
                                    mir::ir::Variable::new("$value")
                                )],
                                None,
                            ),
                            mir::types::Type::Number
                        ),
                        mir::ir::Variable::new("$thunk"),
                    ),
                )
                .into())
            );
        }

        #[test]
        fn compile_spawn_operation_with_any_type() {
            assert_eq!(
                compile_expression(
                    &SpawnOperation::new(
                        Lambda::new(
                            vec![],
                            types::Any::new(Position::fake()),
                            Variable::new("x", Position::fake()),
                            Position::fake()
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Let::new(
                    "$any_thunk",
                    mir::types::Function::new(vec![], mir::types::Type::Variant),
                    mir::ir::Call::new(
                        type_compiler::compile_spawn_function(),
                        mir::ir::Variable::new(MODULE_LOCAL_SPAWN_FUNCTION_NAME),
                        vec![mir::ir::LetRecursive::new(
                            mir::ir::Definition::thunk(
                                "$any_thunk",
                                mir::ir::Variable::new("x"),
                                mir::types::Type::Variant
                            ),
                            mir::ir::Variable::new("$any_thunk"),
                        )
                        .into()]
                    ),
                    mir::ir::LetRecursive::new(
                        mir::ir::Definition::new(
                            "$thunk",
                            vec![],
                            mir::ir::Call::new(
                                mir::types::Function::new(vec![], mir::types::Type::Variant),
                                mir::ir::Variable::new("$any_thunk"),
                                vec![]
                            ),
                            mir::types::Type::Variant
                        ),
                        mir::ir::Variable::new("$thunk"),
                    ),
                )
                .into())
            );
        }
    }
}
