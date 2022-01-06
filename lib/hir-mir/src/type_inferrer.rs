use super::{compile_context::CompileContext, environment_creator, type_extractor, CompileError};
use hir::{
    analysis::types::{type_canonicalizer, type_difference_calculator, union_type_creator},
    ir::*,
    types::{self, Type},
};
use std::collections::BTreeMap;

pub fn infer_types(
    module: &Module,
    compile_context: &CompileContext,
) -> Result<Module, CompileError> {
    let variables = environment_creator::create_from_module(module);

    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| infer_definition(definition, &variables, compile_context))
            .collect::<Result<_, _>>()?,
        module.position().clone(),
    ))
}

fn infer_definition(
    definition: &Definition,
    variables: &BTreeMap<String, Type>,
    compile_context: &CompileContext,
) -> Result<Definition, CompileError> {
    Ok(Definition::new(
        definition.name(),
        definition.original_name(),
        infer_lambda(definition.lambda(), variables, compile_context)?,
        definition.foreign_definition_configuration().cloned(),
        definition.is_public(),
        definition.position().clone(),
    ))
}

fn infer_lambda(
    lambda: &Lambda,
    variables: &BTreeMap<String, Type>,
    compile_context: &CompileContext,
) -> Result<Lambda, CompileError> {
    Ok(Lambda::new(
        lambda.arguments().to_vec(),
        lambda.result_type().clone(),
        infer_expression(
            lambda.body(),
            &variables
                .clone()
                .into_iter()
                .chain(
                    lambda
                        .arguments()
                        .iter()
                        .map(|argument| (argument.name().into(), argument.type_().clone())),
                )
                .collect(),
            compile_context,
        )?,
        lambda.position().clone(),
    ))
}

fn infer_expression(
    expression: &Expression,
    variables: &BTreeMap<String, Type>,
    compile_context: &CompileContext,
) -> Result<Expression, CompileError> {
    let infer_expression =
        |expression, variables: &_| infer_expression(expression, variables, compile_context);

    Ok(match expression {
        Expression::Call(call) => {
            let function = infer_expression(call.function(), variables)?;

            Call::new(
                Some(type_extractor::extract_from_expression(
                    &function,
                    variables,
                    compile_context,
                )?),
                function.clone(),
                call.arguments()
                    .iter()
                    .map(|argument| infer_expression(argument, variables))
                    .collect::<Result<_, _>>()?,
                call.position().clone(),
            )
            .into()
        }
        Expression::If(if_) => {
            let then = infer_expression(if_.then(), variables)?;
            let else_ = infer_expression(if_.else_(), variables)?;

            If::new(
                infer_expression(if_.condition(), variables)?,
                then,
                else_,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfList(if_) => {
            let list = infer_expression(if_.argument(), variables)?;
            let list_type = type_canonicalizer::canonicalize_list(
                &type_extractor::extract_from_expression(&list, variables, compile_context)?,
                compile_context.types(),
            )?
            .ok_or_else(|| CompileError::ListExpected(if_.argument().position().clone()))?;

            let then = infer_expression(
                if_.then(),
                &variables
                    .clone()
                    .into_iter()
                    .chain(vec![
                        (
                            if_.first_name().into(),
                            types::Function::new(
                                vec![],
                                list_type.element().clone(),
                                if_.position().clone(),
                            )
                            .into(),
                        ),
                        (if_.rest_name().into(), list_type.clone().into()),
                    ])
                    .collect(),
            )?;
            let else_ = infer_expression(if_.else_(), variables)?;

            IfList::new(
                Some(list_type.element().clone()),
                list,
                if_.first_name(),
                if_.rest_name(),
                then,
                else_,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfType(if_) => {
            let argument = infer_expression(if_.argument(), variables)?;
            let branches = if_
                .branches()
                .iter()
                .map(|branch| -> Result<_, CompileError> {
                    Ok(IfTypeBranch::new(
                        branch.type_().clone(),
                        infer_expression(
                            branch.expression(),
                            &variables
                                .clone()
                                .into_iter()
                                .chain(vec![(if_.name().into(), branch.type_().clone())])
                                .collect(),
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let else_ = if_
                .else_()
                .map(|branch| -> Result<_, CompileError> {
                    let type_ = type_difference_calculator::calculate(
                        &type_extractor::extract_from_expression(
                            &argument,
                            variables,
                            compile_context,
                        )?,
                        &union_type_creator::create(
                            &if_.branches()
                                .iter()
                                .map(|branch| branch.type_().clone())
                                .collect::<Vec<_>>(),
                            if_.position(),
                        )
                        .unwrap(),
                        compile_context.types(),
                    )?
                    .ok_or_else(|| CompileError::UnreachableCode(branch.position().clone()))?;

                    Ok(ElseBranch::new(
                        Some(type_.clone()),
                        infer_expression(
                            branch.expression(),
                            &variables
                                .clone()
                                .into_iter()
                                .chain(vec![(if_.name().into(), type_)])
                                .collect(),
                        )?,
                        branch.position().clone(),
                    ))
                })
                .transpose()?;

            IfType::new(
                if_.name(),
                argument,
                branches,
                else_,
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => infer_lambda(lambda, variables, compile_context)?.into(),
        Expression::Let(let_) => {
            let bound_expression = infer_expression(let_.bound_expression(), variables)?;
            let bound_type = type_extractor::extract_from_expression(
                &bound_expression,
                variables,
                compile_context,
            )?;

            Let::new(
                let_.name().map(String::from),
                Some(bound_type.clone()),
                bound_expression,
                infer_expression(
                    let_.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain(let_.name().map(|name| (name.into(), bound_type)))
                        .collect(),
                )?,
                let_.position().clone(),
            )
            .into()
        }
        Expression::List(list) => List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| {
                    Ok(match element {
                        ListElement::Multiple(element) => {
                            ListElement::Multiple(infer_expression(element, variables)?)
                        }
                        ListElement::Single(element) => {
                            ListElement::Single(infer_expression(element, variables)?)
                        }
                    })
                })
                .collect::<Result<_, CompileError>>()?,
            list.position().clone(),
        )
        .into(),
        Expression::Operation(operation) => match operation {
            Operation::Arithmetic(operation) => ArithmeticOperation::new(
                operation.operator(),
                infer_expression(operation.lhs(), variables)?,
                infer_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Spawn(operation) => SpawnOperation::new(
                infer_lambda(operation.function(), variables, compile_context)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Boolean(operation) => BooleanOperation::new(
                operation.operator(),
                infer_expression(operation.lhs(), variables)?,
                infer_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Equality(operation) => {
                let lhs = infer_expression(operation.lhs(), variables)?;
                let rhs = infer_expression(operation.rhs(), variables)?;

                EqualityOperation::new(
                    Some(
                        types::Union::new(
                            type_extractor::extract_from_expression(
                                &lhs,
                                variables,
                                compile_context,
                            )?,
                            type_extractor::extract_from_expression(
                                &rhs,
                                variables,
                                compile_context,
                            )?,
                            operation.position().clone(),
                        )
                        .into(),
                    ),
                    operation.operator(),
                    lhs,
                    rhs,
                    operation.position().clone(),
                )
                .into()
            }
            Operation::Not(operation) => NotOperation::new(
                infer_expression(operation.expression(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Order(operation) => OrderOperation::new(
                operation.operator(),
                infer_expression(operation.lhs(), variables)?,
                infer_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Try(operation) => {
                let position = operation.position();
                let expression = infer_expression(operation.expression(), variables)?;
                let error_type = types::Reference::new(
                    &compile_context
                        .compile_configuration()
                        .error_type
                        .error_type_name,
                    position.clone(),
                )
                .into();

                TryOperation::new(
                    Some(
                        if let Some(type_) = type_difference_calculator::calculate(
                            &type_extractor::extract_from_expression(
                                &expression,
                                variables,
                                compile_context,
                            )?,
                            &error_type,
                            compile_context.types(),
                        )? {
                            if type_.is_any() {
                                return Err(CompileError::UnionTypeExpected(
                                    expression.position().clone(),
                                ));
                            } else {
                                type_
                            }
                        } else {
                            return Err(CompileError::UnionTypeExpected(
                                expression.position().clone(),
                            ));
                        },
                    ),
                    expression,
                    position.clone(),
                )
                .into()
            }
        },
        Expression::RecordConstruction(construction) => RecordConstruction::new(
            construction.type_().clone(),
            construction
                .fields()
                .iter()
                .map(|field| {
                    Ok(RecordField::new(
                        field.name(),
                        infer_expression(field.expression(), variables)?,
                        field.position().clone(),
                    ))
                })
                .collect::<Result<_, CompileError>>()?,
            construction.position().clone(),
        )
        .into(),
        Expression::RecordDeconstruction(deconstruction) => {
            let record = infer_expression(deconstruction.record(), variables)?;

            RecordDeconstruction::new(
                Some(type_extractor::extract_from_expression(
                    &record,
                    variables,
                    compile_context,
                )?),
                record,
                deconstruction.field_name(),
                deconstruction.position().clone(),
            )
            .into()
        }
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            infer_expression(update.record(), variables)?,
            update
                .fields()
                .iter()
                .map(|field| {
                    Ok(RecordField::new(
                        field.name(),
                        infer_expression(field.expression(), variables)?,
                        field.position().clone(),
                    ))
                })
                .collect::<Result<_, CompileError>>()?,
            update.position().clone(),
        )
        .into(),
        Expression::Thunk(thunk) => Thunk::new(
            Some(type_extractor::extract_from_expression(
                thunk.expression(),
                variables,
                compile_context,
            )?),
            infer_expression(thunk.expression(), variables)?,
            thunk.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => TypeCoercion::new(
            coercion.from().clone(),
            coercion.to().clone(),
            infer_expression(coercion.argument(), variables)?,
            coercion.position().clone(),
        )
        .into(),
        Expression::Boolean(_)
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Variable(_) => expression.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::test::{DefinitionFake, ModuleFake, TypeDefinitionFake};
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn infer_module(module: &Module) -> Result<Module, CompileError> {
        infer_types(
            module,
            &CompileContext::new(module, COMPILE_CONFIGURATION.clone()),
        )
    }

    #[test]
    fn infer_empty_module() {
        infer_module(&Module::empty()).unwrap();
    }

    #[test]
    fn infer_call() {
        assert_eq!(
            infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    Call::new(
                        None,
                        Variable::new("x", Position::fake()),
                        vec![],
                        Position::fake()
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    Call::new(
                        Some(
                            types::Function::new(
                                vec![],
                                types::None::new(Position::fake()),
                                Position::fake()
                            )
                            .into()
                        ),
                        Variable::new("x", Position::fake()),
                        vec![],
                        Position::fake()
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn infer_equality_operation() {
        assert_eq!(
            infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    EqualityOperation::new(
                        None,
                        EqualityOperator::Equal,
                        None::new(Position::fake()),
                        None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    EqualityOperation::new(
                        Some(
                            types::Union::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake()
                            )
                            .into()
                        ),
                        EqualityOperator::Equal,
                        None::new(Position::fake()),
                        None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn infer_let() {
        assert_eq!(
            infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    Let::new(
                        Some("x".into()),
                        None,
                        None::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    Let::new(
                        Some("x".into()),
                        Some(types::None::new(Position::fake()).into()),
                        None::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn infer_let_with_call() {
        let declaration = Declaration::new(
            "f",
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            infer_module(
                &Module::empty()
                    .set_declarations(vec![declaration.clone()])
                    .set_definitions(vec![Definition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Let::new(
                                Some("x".into()),
                                None,
                                Call::new(
                                    None,
                                    Variable::new("f", Position::fake()),
                                    vec![],
                                    Position::fake()
                                ),
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
            ),
            Ok(Module::empty()
                .set_declarations(vec![declaration.clone()])
                .set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Let::new(
                            Some("x".into()),
                            Some(types::None::new(Position::fake()).into()),
                            Call::new(
                                Some(declaration.type_().clone().into()),
                                Variable::new("f", Position::fake()),
                                vec![],
                                Position::fake()
                            ),
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]))
        );
    }

    #[test]
    fn infer_record_deconstruction() {
        let type_definition = TypeDefinition::new(
            "r",
            "",
            vec![types::RecordField::new(
                "x",
                types::None::new(Position::fake()),
            )],
            false,
            false,
            false,
            Position::fake(),
        );

        assert_eq!(
            infer_module(
                &Module::empty()
                    .set_type_definitions(vec![type_definition.clone()])
                    .set_definitions(vec![Definition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Record::new("r", Position::fake())
                            )],
                            types::None::new(Position::fake()),
                            RecordDeconstruction::new(
                                None,
                                Variable::new("x", Position::fake()),
                                "x",
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
            ),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Record::new("r", Position::fake())
                        )],
                        types::None::new(Position::fake()),
                        RecordDeconstruction::new(
                            Some(types::Record::new("r", Position::fake()).into()),
                            Variable::new("x", Position::fake()),
                            "x",
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )]))
        );
    }

    #[test]
    fn infer_thunk() {
        let none_type = types::None::new(Position::fake());

        assert_eq!(
            infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    none_type.clone(),
                    Thunk::new(None, None::new(Position::fake()), Position::fake()),
                    Position::fake(),
                ),
                false,
            )])),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    none_type.clone(),
                    Thunk::new(
                        Some(none_type.into()),
                        None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                ),
                false,
            )]))
        );
    }

    mod if_type {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn infer_else_branch_type_of_none() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let branches = vec![IfTypeBranch::new(
                types::Number::new(Position::fake()),
                None::new(Position::fake()),
            )];

            assert_eq!(
                infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", union_type.clone())],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "x",
                            Variable::new("x", Position::fake()),
                            branches.clone(),
                            Some(ElseBranch::new(
                                None,
                                None::new(Position::fake()),
                                Position::fake()
                            )),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],)),
                Ok(Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", union_type)],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "x",
                            Variable::new("x", Position::fake()),
                            branches,
                            Some(ElseBranch::new(
                                Some(types::None::new(Position::fake()).into()),
                                None::new(Position::fake()),
                                Position::fake()
                            )),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],))
            );
        }

        #[test]
        fn infer_else_branch_type_of_union() {
            let union_type = types::Union::new(
                types::Union::new(
                    types::Number::new(Position::fake()),
                    types::Boolean::new(Position::fake()),
                    Position::fake(),
                ),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let branches = vec![IfTypeBranch::new(
                types::Number::new(Position::fake()),
                None::new(Position::fake()),
            )];

            assert_eq!(
                infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", union_type.clone())],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "x",
                            Variable::new("x", Position::fake()),
                            branches.clone(),
                            Some(ElseBranch::new(
                                None,
                                None::new(Position::fake()),
                                Position::fake()
                            )),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],)),
                Ok(Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", union_type)],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "x",
                            Variable::new("x", Position::fake()),
                            branches,
                            Some(ElseBranch::new(
                                Some(
                                    types::Union::new(
                                        types::Boolean::new(Position::fake()),
                                        types::None::new(Position::fake()),
                                        Position::fake(),
                                    )
                                    .into()
                                ),
                                None::new(Position::fake()),
                                Position::fake()
                            )),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],))
            );
        }

        #[test]
        fn infer_else_branch_type_with_bound_variable() {
            let function_type =
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());
            let union_type = types::Union::new(
                function_type.clone(),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let branches = vec![IfTypeBranch::new(
                types::None::new(Position::fake()),
                None::new(Position::fake()),
            )];

            assert_eq!(
                infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", union_type.clone())],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            branches.clone(),
                            Some(ElseBranch::new(
                                None,
                                Call::new(
                                    None,
                                    Variable::new("y", Position::fake()),
                                    vec![],
                                    Position::fake()
                                ),
                                Position::fake()
                            )),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],)),
                Ok(Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", union_type)],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            branches,
                            Some(ElseBranch::new(
                                Some(function_type.clone().into()),
                                Call::new(
                                    Some(function_type.into()),
                                    Variable::new("y", Position::fake()),
                                    vec![],
                                    Position::fake()
                                ),
                                Position::fake()
                            )),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],))
            );
        }

        #[test]
        fn infer_else_branch_type_of_any() {
            let any_type = types::Any::new(Position::fake());
            let branches = vec![IfTypeBranch::new(
                types::Number::new(Position::fake()),
                None::new(Position::fake()),
            )];

            assert_eq!(
                infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", any_type.clone())],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "x",
                            Variable::new("x", Position::fake()),
                            branches.clone(),
                            Some(ElseBranch::new(
                                None,
                                None::new(Position::fake()),
                                Position::fake()
                            )),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],)),
                Ok(Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", any_type.clone())],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "x",
                            Variable::new("x", Position::fake()),
                            branches,
                            Some(ElseBranch::new(
                                Some(any_type.into()),
                                None::new(Position::fake()),
                                Position::fake()
                            )),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],))
            );
        }

        #[test]
        fn fail_to_infer_else_branch_type_due_to_unreachable_code() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", union_type)],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "x",
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
                            Some(ElseBranch::new(
                                None,
                                None::new(Position::fake()),
                                Position::fake()
                            )),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],)),
                Err(CompileError::UnreachableCode(Position::fake()))
            );
        }
    }

    mod try_operation {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn infer() {
            let union_type = types::Union::new(
                types::None::new(Position::fake()),
                types::Reference::new("error", Position::fake()),
                Position::fake(),
            );
            let module = Module::empty().set_type_definitions(vec![TypeDefinition::fake(
                "error",
                vec![],
                false,
                false,
                false,
            )]);

            assert_eq!(
                infer_module(&module.set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", union_type.clone())],
                        union_type.clone(),
                        TryOperation::new(
                            None,
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )])),
                Ok(module.set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", union_type.clone())],
                        union_type,
                        TryOperation::new(
                            Some(types::None::new(Position::fake()).into()),
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )],))
            );
        }

        #[test]
        fn fail_to_infer_with_error() {
            let error_type = types::Reference::new("error", Position::fake());

            assert_eq!(
                infer_module(
                    &Module::empty()
                        .set_type_definitions(vec![TypeDefinition::fake(
                            "error",
                            vec![],
                            false,
                            false,
                            false,
                        )])
                        .set_definitions(vec![Definition::fake(
                            "f",
                            Lambda::new(
                                vec![Argument::new("x", error_type.clone())],
                                error_type,
                                TryOperation::new(
                                    None,
                                    Variable::new("x", Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            false,
                        )],)
                ),
                Err(CompileError::UnionTypeExpected(Position::fake()))
            );
        }
    }

    mod if_list {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn infer() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        types::None::new(Position::fake()),
                        IfList::new(
                            None,
                            Variable::new("x", Position::fake()),
                            "y",
                            "ys",
                            Variable::new("y", Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )])),
                Ok(Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", list_type)],
                        types::None::new(Position::fake()),
                        IfList::new(
                            Some(types::None::new(Position::fake()).into()),
                            Variable::new("x", Position::fake()),
                            "y",
                            "ys",
                            Variable::new("y", Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )],))
            );
        }

        #[test]
        fn infer_with_first_name_in_let() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        types::None::new(Position::fake()),
                        IfList::new(
                            None,
                            Variable::new("x", Position::fake()),
                            "y",
                            "ys",
                            Let::new(
                                Some("z".into()),
                                None,
                                Variable::new("y", Position::fake()),
                                Variable::new("z", Position::fake()),
                                Position::fake()
                            ),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )])),
                Ok(Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", list_type)],
                        types::None::new(Position::fake()),
                        IfList::new(
                            Some(types::None::new(Position::fake()).into()),
                            Variable::new("x", Position::fake()),
                            "y",
                            "ys",
                            Let::new(
                                Some("z".into()),
                                Some(
                                    types::Function::new(
                                        vec![],
                                        types::None::new(Position::fake()),
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                Variable::new("y", Position::fake()),
                                Variable::new("z", Position::fake()),
                                Position::fake()
                            ),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )],))
            );
        }

        #[test]
        fn infer_with_rest_name_in_let() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        types::None::new(Position::fake()),
                        IfList::new(
                            None,
                            Variable::new("x", Position::fake()),
                            "y",
                            "ys",
                            Let::new(
                                Some("z".into()),
                                None,
                                Variable::new("ys", Position::fake()),
                                Variable::new("z", Position::fake()),
                                Position::fake()
                            ),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )])),
                Ok(Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", list_type)],
                        types::None::new(Position::fake()),
                        IfList::new(
                            Some(types::None::new(Position::fake()).into()),
                            Variable::new("x", Position::fake()),
                            "y",
                            "ys",
                            Let::new(
                                Some("z".into()),
                                Some(
                                    types::List::new(
                                        types::None::new(Position::fake()),
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                Variable::new("ys", Position::fake()),
                                Variable::new("z", Position::fake()),
                                Position::fake()
                            ),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )],))
            );
        }
    }
}
