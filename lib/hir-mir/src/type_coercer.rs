use super::{environment_creator, type_context::TypeContext, type_extractor, CompileError};
use hir::{
    analysis::types::{record_field_resolver, type_canonicalizer, type_equality_checker},
    ir::*,
    types::{self, Type},
};
use position::Position;
use std::collections::BTreeMap;

pub fn coerce_types(module: &Module, type_context: &TypeContext) -> Result<Module, CompileError> {
    let variables = environment_creator::create_from_module(module);

    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| transform_definition(definition, &variables, type_context))
            .collect::<Result<_, _>>()?,
        module.position().clone(),
    ))
}

fn transform_definition(
    definition: &Definition,
    variables: &BTreeMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Definition, CompileError> {
    Ok(Definition::new(
        definition.name(),
        definition.original_name(),
        transform_lambda(definition.lambda(), variables, type_context)?,
        definition.foreign_definition_configuration().cloned(),
        definition.is_public(),
        definition.position().clone(),
    ))
}

fn transform_lambda(
    lambda: &Lambda,
    variables: &BTreeMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Lambda, CompileError> {
    let variables = variables
        .clone()
        .into_iter()
        .chain(
            lambda
                .arguments()
                .iter()
                .map(|argument| (argument.name().into(), argument.type_().clone())),
        )
        .collect();

    Ok(Lambda::new(
        lambda.arguments().to_vec(),
        lambda.result_type().clone(),
        coerce_expression(
            &transform_expression(lambda.body(), &variables, type_context)?,
            lambda.result_type(),
            &variables,
            type_context,
        )?,
        lambda.position().clone(),
    ))
}

fn transform_expression(
    expression: &Expression,
    variables: &BTreeMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Expression, CompileError> {
    let transform_expression =
        |expression, variables: &_| transform_expression(expression, variables, type_context);
    let transform_and_coerce_expression = |expression, type_: &_, variables: &_| {
        coerce_expression(
            &transform_expression(expression, variables)?,
            type_,
            variables,
            type_context,
        )
    };
    let extract_type = |expression, variables| {
        type_extractor::extract_from_expression(expression, variables, type_context)
    };

    Ok(match expression {
        Expression::Call(call) => {
            let function_type = type_canonicalizer::canonicalize_function(
                call.function_type()
                    .ok_or_else(|| CompileError::TypeNotInferred(call.position().clone()))?,
                type_context.types(),
            )?
            .ok_or_else(|| CompileError::FunctionExpected(call.position().clone()))?;

            Call::new(
                call.function_type().cloned(),
                transform_expression(call.function(), variables)?,
                call.arguments()
                    .iter()
                    .zip(function_type.arguments())
                    .map(|(argument, type_)| {
                        transform_and_coerce_expression(argument, type_, variables)
                    })
                    .collect::<Result<_, _>>()?,
                call.position().clone(),
            )
            .into()
        }
        Expression::If(if_) => {
            let type_ = extract_type(expression, variables)?;

            If::new(
                transform_expression(if_.condition(), variables)?,
                transform_and_coerce_expression(if_.then(), &type_, variables)?,
                transform_and_coerce_expression(if_.else_(), &type_, variables)?,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfList(if_) => {
            let list_type = types::List::new(
                if_.type_()
                    .ok_or_else(|| {
                        CompileError::TypeNotInferred(if_.argument().position().clone())
                    })?
                    .clone(),
                if_.argument().position().clone(),
            );
            let result_type = extract_type(expression, variables)?;

            IfList::new(
                if_.type_().cloned(),
                transform_expression(if_.argument(), variables)?,
                if_.first_name(),
                if_.rest_name(),
                transform_and_coerce_expression(
                    if_.then(),
                    &result_type,
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
                )?,
                transform_and_coerce_expression(if_.else_(), &result_type, variables)?,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfType(if_) => {
            let result_type = extract_type(expression, variables)?;

            IfType::new(
                if_.name(),
                transform_expression(if_.argument(), variables)?,
                if_.branches()
                    .iter()
                    .map(|branch| -> Result<_, CompileError> {
                        Ok(IfTypeBranch::new(
                            branch.type_().clone(),
                            transform_and_coerce_expression(
                                branch.expression(),
                                &result_type,
                                &variables
                                    .clone()
                                    .into_iter()
                                    .chain(vec![(if_.name().into(), branch.type_().clone())])
                                    .collect(),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                if_.else_()
                    .map(|branch| -> Result<_, CompileError> {
                        Ok(ElseBranch::new(
                            branch.type_().cloned(),
                            transform_and_coerce_expression(
                                branch.expression(),
                                &result_type,
                                &variables
                                    .clone()
                                    .into_iter()
                                    .chain(vec![(
                                        if_.name().into(),
                                        branch
                                            .type_()
                                            .ok_or_else(|| {
                                                CompileError::TypeNotInferred(
                                                    branch.position().clone(),
                                                )
                                            })?
                                            .clone(),
                                    )])
                                    .collect(),
                            )?,
                            branch.position().clone(),
                        ))
                    })
                    .transpose()?,
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => transform_lambda(lambda, variables, type_context)?.into(),
        Expression::Let(let_) => Let::new(
            let_.name().map(String::from),
            let_.type_().cloned(),
            transform_expression(let_.bound_expression(), variables)?,
            transform_expression(
                let_.expression(),
                &variables
                    .clone()
                    .into_iter()
                    .chain(
                        let_.name()
                            .map(|name| -> Result<_, CompileError> {
                                Ok((
                                    name.into(),
                                    let_.type_()
                                        .ok_or_else(|| {
                                            CompileError::TypeNotInferred(
                                                let_.bound_expression().position().clone(),
                                            )
                                        })?
                                        .clone(),
                                ))
                            })
                            .transpose()?,
                    )
                    .collect(),
            )?,
            let_.position().clone(),
        )
        .into(),
        Expression::List(list) => List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| {
                    Ok(match element {
                        ListElement::Multiple(element) => {
                            ListElement::Multiple(transform_and_coerce_expression(
                                element,
                                &types::List::new(list.type_().clone(), element.position().clone())
                                    .into(),
                                variables,
                            )?)
                        }
                        ListElement::Single(element) => ListElement::Single(
                            transform_and_coerce_expression(element, list.type_(), variables)?,
                        ),
                    })
                })
                .collect::<Result<_, CompileError>>()?,
            list.position().clone(),
        )
        .into(),
        Expression::Operation(operation) => match operation {
            Operation::Arithmetic(operation) => ArithmeticOperation::new(
                operation.operator(),
                transform_expression(operation.lhs(), variables)?,
                transform_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Boolean(operation) => BooleanOperation::new(
                operation.operator(),
                transform_expression(operation.lhs(), variables)?,
                transform_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Equality(operation) => {
                let type_ = operation
                    .type_()
                    .ok_or_else(|| CompileError::TypeNotInferred(operation.position().clone()))?;

                EqualityOperation::new(
                    operation.type_().cloned(),
                    operation.operator(),
                    transform_and_coerce_expression(operation.lhs(), type_, variables)?,
                    transform_and_coerce_expression(operation.rhs(), type_, variables)?,
                    operation.position().clone(),
                )
                .into()
            }
            Operation::Not(operation) => NotOperation::new(
                transform_expression(operation.expression(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Order(operation) => OrderOperation::new(
                operation.operator(),
                transform_expression(operation.lhs(), variables)?,
                transform_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Try(operation) => TryOperation::new(
                operation.type_().cloned(),
                transform_expression(operation.expression(), variables)?,
                operation.position().clone(),
            )
            .into(),
        },
        Expression::RecordConstruction(construction) => RecordConstruction::new(
            construction.type_().clone(),
            transform_record_fields(
                construction.fields(),
                construction.position(),
                construction.type_(),
                variables,
                type_context,
            )?,
            construction.position().clone(),
        )
        .into(),
        Expression::RecordDeconstruction(deconstruction) => RecordDeconstruction::new(
            deconstruction.type_().cloned(),
            transform_expression(deconstruction.record(), variables)?,
            deconstruction.field_name(),
            deconstruction.position().clone(),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            transform_expression(update.record(), variables)?,
            transform_record_fields(
                update.fields(),
                update.position(),
                update.type_(),
                variables,
                type_context,
            )?,
            update.position().clone(),
        )
        .into(),
        Expression::Thunk(thunk) => Thunk::new(
            thunk.type_().cloned(),
            transform_and_coerce_expression(
                thunk.expression(),
                thunk
                    .type_()
                    .ok_or_else(|| CompileError::TypeNotInferred(thunk.position().clone()))?,
                variables,
            )?,
            thunk.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => TypeCoercion::new(
            coercion.from().clone(),
            coercion.to().clone(),
            transform_expression(coercion.argument(), variables)?,
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

fn transform_record_fields(
    fields: &[RecordField],
    position: &Position,
    record_type: &Type,
    variables: &BTreeMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Vec<RecordField>, CompileError> {
    let field_types = record_field_resolver::resolve(
        record_type,
        position,
        type_context.types(),
        type_context.records(),
    )?;

    fields
        .iter()
        .map(|field| {
            Ok(RecordField::new(
                field.name(),
                coerce_expression(
                    &transform_expression(field.expression(), variables, type_context)?,
                    field_types
                        .iter()
                        .find(|field_type| field_type.name() == field.name())
                        .ok_or_else(|| CompileError::RecordFieldUnknown(field.position().clone()))?
                        .type_(),
                    variables,
                    type_context,
                )?,
                field.position().clone(),
            ))
        })
        .collect::<Result<_, _>>()
}

fn coerce_expression(
    expression: &Expression,
    upper_type: &Type,
    variables: &BTreeMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Expression, CompileError> {
    let lower_type = type_extractor::extract_from_expression(expression, variables, type_context)?;

    Ok(
        if type_equality_checker::check(&lower_type, upper_type, type_context.types())? {
            expression.clone()
        } else {
            TypeCoercion::new(
                lower_type,
                upper_type.clone(),
                expression.clone(),
                expression.position().clone(),
            )
            .into()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error_type_configuration::ERROR_TYPE_CONFIGURATION,
        list_type_configuration::LIST_TYPE_CONFIGURATION,
        string_type_configuration::STRING_TYPE_CONFIGURATION,
    };
    use hir::test::{DefinitionFake, ModuleFake, TypeDefinitionFake};
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn coerce_module(module: &Module) -> Result<Module, CompileError> {
        coerce_types(
            module,
            &TypeContext::new(
                module,
                &LIST_TYPE_CONFIGURATION,
                &STRING_TYPE_CONFIGURATION,
                &ERROR_TYPE_CONFIGURATION,
            ),
        )
    }

    #[test]
    fn coerce_function_result() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    union_type.clone(),
                    None::new(Position::fake()),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    union_type.clone(),
                    TypeCoercion::new(
                        types::None::new(Position::fake()),
                        union_type,
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
    fn coerce_function_result_of_variable() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("x", types::None::new(Position::fake()))],
                    union_type.clone(),
                    Variable::new("x", Position::fake()),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("x", types::None::new(Position::fake()))],
                    union_type.clone(),
                    TypeCoercion::new(
                        types::None::new(Position::fake()),
                        union_type,
                        Variable::new("x", Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn coerce_if() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    union_type.clone(),
                    If::new(
                        Boolean::new(true, Position::fake()),
                        Number::new(42.0, Position::fake()),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    union_type.clone(),
                    If::new(
                        Boolean::new(true, Position::fake()),
                        TypeCoercion::new(
                            types::Number::new(Position::fake()),
                            union_type.clone(),
                            Number::new(42.0, Position::fake()),
                            Position::fake(),
                        ),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            union_type,
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn coerce_if_list() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let list_type = types::List::new(types::Number::new(Position::fake()), Position::fake());
        let element_call = Call::new(
            Some(
                types::Function::new(
                    vec![],
                    types::Number::new(Position::fake()),
                    Position::fake(),
                )
                .into(),
            ),
            Variable::new("x", Position::fake()),
            vec![],
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("xs", list_type.clone())],
                    union_type.clone(),
                    IfList::new(
                        Some(types::Number::new(Position::fake()).into()),
                        Variable::new("xs", Position::fake()),
                        "x",
                        "xs",
                        element_call.clone(),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("xs", list_type)],
                    union_type.clone(),
                    IfList::new(
                        Some(types::Number::new(Position::fake()).into()),
                        Variable::new("xs", Position::fake()),
                        "x",
                        "xs",
                        TypeCoercion::new(
                            types::Number::new(Position::fake()),
                            union_type.clone(),
                            element_call,
                            Position::fake(),
                        ),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            union_type,
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn coerce_if_list_with_function_union_type() {
        let number_thunk_type = types::Function::new(
            vec![],
            types::Number::new(Position::fake()),
            Position::fake(),
        );
        let union_type = types::Union::new(
            number_thunk_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let list_type = types::List::new(types::Number::new(Position::fake()), Position::fake());

        assert_eq!(
            coerce_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("xs", list_type.clone())],
                    union_type.clone(),
                    IfList::new(
                        Some(types::Number::new(Position::fake()).into()),
                        Variable::new("xs", Position::fake()),
                        "x",
                        "xs",
                        Variable::new("x", Position::fake()),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("xs", list_type)],
                    union_type.clone(),
                    IfList::new(
                        Some(types::Number::new(Position::fake()).into()),
                        Variable::new("xs", Position::fake()),
                        "x",
                        "xs",
                        TypeCoercion::new(
                            number_thunk_type,
                            union_type.clone(),
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            union_type,
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn coerce_if_type() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("x", union_type.clone())],
                    union_type.clone(),
                    IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Number::new(Position::fake()),
                            Variable::new("y", Position::fake()),
                        )],
                        Some(ElseBranch::new(
                            Some(types::None::new(Position::fake()).into()),
                            None::new(Position::fake()),
                            Position::fake(),
                        )),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("x", union_type.clone())],
                    union_type.clone(),
                    IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Number::new(Position::fake()),
                            TypeCoercion::new(
                                types::Number::new(Position::fake()),
                                union_type.clone(),
                                Variable::new("y", Position::fake()),
                                Position::fake(),
                            ),
                        )],
                        Some(ElseBranch::new(
                            Some(types::None::new(Position::fake()).into()),
                            TypeCoercion::new(
                                types::None::new(Position::fake()),
                                union_type,
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake()
                        )),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn coerce_equality_operation() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::Boolean::new(Position::fake()),
                    EqualityOperation::new(
                        Some(union_type.clone().into()),
                        EqualityOperator::Equal,
                        Number::new(42.0, Position::fake()),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::Boolean::new(Position::fake()),
                    EqualityOperation::new(
                        Some(union_type.clone().into()),
                        EqualityOperator::Equal,
                        TypeCoercion::new(
                            types::Number::new(Position::fake()),
                            union_type.clone(),
                            Number::new(42.0, Position::fake()),
                            Position::fake(),
                        ),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            union_type,
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn coerce_single_element_in_list() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let list_type = types::List::new(union_type.clone(), Position::fake());

        assert_eq!(
            coerce_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    list_type.clone(),
                    List::new(
                        union_type.clone(),
                        vec![ListElement::Single(None::new(Position::fake()).into())],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    list_type,
                    List::new(
                        union_type.clone(),
                        vec![ListElement::Single(
                            TypeCoercion::new(
                                types::None::new(Position::fake()),
                                union_type,
                                None::new(Position::fake()),
                                Position::fake(),
                            )
                            .into()
                        )],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn coerce_multiple_element_in_list() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let list_type = types::List::new(union_type.clone(), Position::fake());

        assert_eq!(
            coerce_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    list_type.clone(),
                    List::new(
                        union_type.clone(),
                        vec![ListElement::Multiple(
                                List::new(
                                    types::None::new(Position::fake()),
                                    vec![],
                                    Position::fake()
                                )
                                .into()
                            )],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    list_type.clone(),
                    List::new(
                        union_type,
                        vec![ListElement::Multiple(
                            TypeCoercion::new(
                                types::List::new(
                                    types::None::new(Position::fake()),
                                    Position::fake()
                                ),
                                list_type,
                                List::new(
                                    types::None::new(Position::fake()),
                                    vec![],
                                    Position::fake()
                                ),
                                Position::fake(),
                            )
                            .into()
                        )],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }

    #[test]
    fn coerce_record_construction() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let type_definition = TypeDefinition::fake(
            "r",
            vec![types::RecordField::new("x", union_type.clone())],
            false,
            false,
            false,
        );
        let record_type = types::Record::new("r", Position::fake());

        assert_eq!(
            coerce_module(
                &Module::empty()
                    .set_type_definitions(vec![type_definition.clone()])
                    .set_definitions(vec![Definition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            record_type.clone(),
                            RecordConstruction::new(
                                record_type.clone(),
                                vec![RecordField::new(
                                    "x",
                                    None::new(Position::fake()),
                                    Position::fake()
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
            ),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        record_type.clone(),
                        RecordConstruction::new(
                            record_type,
                            vec![RecordField::new(
                                "x",
                                TypeCoercion::new(
                                    types::None::new(Position::fake()),
                                    union_type,
                                    None::new(Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]))
        );
    }

    #[test]
    fn coerce_record_update() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let type_definition = TypeDefinition::fake(
            "r",
            vec![types::RecordField::new("x", union_type.clone())],
            false,
            false,
            false,
        );
        let record_type = types::Record::new("r", Position::fake());

        assert_eq!(
            coerce_module(
                &Module::empty()
                    .set_type_definitions(vec![type_definition.clone()])
                    .set_definitions(vec![Definition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("r", record_type.clone())],
                            record_type.clone(),
                            RecordUpdate::new(
                                record_type.clone(),
                                Variable::new("r", Position::fake()),
                                vec![RecordField::new(
                                    "x",
                                    None::new(Position::fake()),
                                    Position::fake()
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
            ),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("r", record_type.clone())],
                        record_type.clone(),
                        RecordUpdate::new(
                            record_type,
                            Variable::new("r", Position::fake()),
                            vec![RecordField::new(
                                "x",
                                TypeCoercion::new(
                                    types::None::new(Position::fake()),
                                    union_type,
                                    None::new(Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]))
        );
    }

    #[test]
    fn coerce_thunk() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::Function::new(vec![], union_type.clone(), Position::fake()),
                    Thunk::new(
                        Some(union_type.clone().into()),
                        None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                ),
                false,
            )],)),
            Ok(Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::Function::new(vec![], union_type.clone(), Position::fake()),
                    Thunk::new(
                        Some(union_type.clone().into()),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            union_type,
                            None::new(Position::fake()),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    Position::fake(),
                ),
                false,
            )],))
        );
    }
}
