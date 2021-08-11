use super::{type_compiler, type_context::TypeContext, CompileError};
use crate::{
    hir::*,
    types::{
        analysis::{type_canonicalizer, TypeError},
        Type,
    },
};
use std::collections::{HashMap, HashSet};

pub fn compile(
    module: &Module,
    type_context: &TypeContext,
) -> Result<Vec<mir::ir::TypeDefinition>, CompileError> {
    Ok(collect_from_module(module, type_context.types())?
        .into_iter()
        .map(|type_| compile_type_definition(&type_, type_context))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn compile_type_definition(
    type_: &Type,
    type_context: &TypeContext,
) -> Result<Option<mir::ir::TypeDefinition>, CompileError> {
    Ok(match type_ {
        Type::List(list_type) => Some(mir::ir::TypeDefinition::new(
            type_compiler::compile_concrete_list_name(list_type, type_context.types())?,
            mir::types::RecordBody::new(vec![mir::types::Record::new(
                &type_context.list_type_configuration().list_type_name,
            )
            .into()]),
        )),
        _ => None,
    })
}

// TODO Generalize this logic into an expression transformer.
fn collect_from_module(
    module: &Module,
    types: &HashMap<String, Type>,
) -> Result<HashSet<Type>, TypeError> {
    module
        .definitions()
        .iter()
        .flat_map(collect_from_definition)
        .map(|type_| type_canonicalizer::canonicalize(&type_, types))
        .collect()
}

fn collect_from_definition(definition: &Definition) -> HashSet<Type> {
    collect_from_expression(definition.lambda().body())
}

fn collect_from_expression(expression: &Expression) -> HashSet<Type> {
    match expression {
        Expression::Call(call) => collect_from_expression(call.function())
            .into_iter()
            .chain(call.arguments().iter().flat_map(collect_from_expression))
            .collect(),
        Expression::If(if_) => collect_from_expression(if_.condition())
            .into_iter()
            .chain(collect_from_expression(if_.then()))
            .chain(collect_from_expression(if_.else_()))
            .collect(),
        Expression::IfList(if_) => collect_from_expression(if_.argument())
            .into_iter()
            .chain(collect_from_expression(if_.then()))
            .chain(collect_from_expression(if_.else_()))
            .collect(),
        Expression::IfType(if_) => if_
            .branches()
            .iter()
            .map(|branch| branch.type_())
            .chain(if_.else_().map(|branch| branch.type_()).flatten())
            .cloned()
            .chain(collect_from_expression(if_.argument()))
            .chain(
                if_.branches()
                    .iter()
                    .flat_map(|branch| collect_from_expression(branch.expression())),
            )
            .chain(
                if_.else_()
                    .into_iter()
                    .flat_map(|branch| collect_from_expression(branch.expression())),
            )
            .collect(),
        Expression::TypeCoercion(coercion) => vec![coercion.from().clone()]
            .into_iter()
            .chain(collect_from_expression(coercion.argument()))
            .collect(),
        Expression::Lambda(lambda) => collect_from_expression(lambda.body()),
        Expression::Let(let_) => collect_from_expression(let_.bound_expression())
            .into_iter()
            .chain(collect_from_expression(let_.expression()))
            .collect(),
        Expression::List(list) => list
            .elements()
            .iter()
            .flat_map(|element| {
                collect_from_expression(match element {
                    ListElement::Multiple(expression) => expression,
                    ListElement::Single(expression) => expression,
                })
            })
            .collect(),
        Expression::Operation(operation) => match operation {
            Operation::Arithmetic(operation) => collect_from_expression(operation.lhs())
                .into_iter()
                .chain(collect_from_expression(operation.rhs()))
                .collect(),
            Operation::Boolean(operation) => collect_from_expression(operation.lhs())
                .into_iter()
                .chain(collect_from_expression(operation.rhs()))
                .collect(),
            Operation::Equality(operation) => collect_from_expression(operation.lhs())
                .into_iter()
                .chain(collect_from_expression(operation.rhs()))
                .collect(),
            Operation::Not(operation) => collect_from_expression(operation.expression()),
            Operation::Order(operation) => collect_from_expression(operation.lhs())
                .into_iter()
                .chain(collect_from_expression(operation.rhs()))
                .collect(),
            Operation::Try(operation) => operation
                .type_()
                .cloned()
                .into_iter()
                .chain(collect_from_expression(operation.expression()))
                .collect(),
        },
        Expression::RecordConstruction(construction) => construction
            .elements()
            .iter()
            .flat_map(|element| collect_from_expression(element.expression()))
            .collect(),
        Expression::RecordDeconstruction(deconstruction) => {
            collect_from_expression(deconstruction.record())
        }
        Expression::RecordUpdate(update) => collect_from_expression(update.record())
            .into_iter()
            .chain(
                update
                    .elements()
                    .iter()
                    .flat_map(|element| collect_from_expression(element.expression())),
            )
            .collect(),
        Expression::Boolean(_)
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Variable(_) => Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position::Position, types};

    #[test]
    fn compile_list_type_definition() {
        let list_type = types::List::new(types::None::new(Position::dummy()), Position::dummy());
        let union_type = types::Union::new(
            list_type.clone(),
            types::None::new(Position::dummy()),
            Position::dummy(),
        );
        let type_context = TypeContext::dummy(Default::default(), Default::default());

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![Definition::without_source(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        types::None::new(Position::dummy()),
                        TypeCoercion::new(
                            list_type.clone(),
                            union_type,
                            Variable::new("x", Position::dummy()),
                            Position::dummy()
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
                &type_context,
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_compiler::compile_concrete_list_name(&list_type, type_context.types())
                    .unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &type_context.list_type_configuration().list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn compile_duplicate_list_type_definitions() {
        let list_type = types::List::new(types::None::new(Position::dummy()), Position::dummy());
        let union_type = types::Union::new(
            list_type.clone(),
            types::None::new(Position::dummy()),
            Position::dummy(),
        );
        let type_context = TypeContext::dummy(Default::default(), Default::default());
        let definition = Definition::without_source(
            "foo",
            Lambda::new(
                vec![Argument::new("x", list_type.clone())],
                types::None::new(Position::dummy()),
                TypeCoercion::new(
                    list_type.clone(),
                    union_type,
                    Variable::new("x", Position::dummy()),
                    Position::dummy(),
                ),
                Position::dummy(),
            ),
            false,
        );

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![definition.clone(), definition]),
                &type_context,
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_compiler::compile_concrete_list_name(&list_type, type_context.types())
                    .unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &type_context.list_type_configuration().list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_if_type() {
        let list_type = types::List::new(types::None::new(Position::dummy()), Position::dummy());
        let type_context = TypeContext::dummy(Default::default(), Default::default());

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![Definition::without_source(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        types::None::new(Position::dummy()),
                        IfType::new(
                            "x",
                            Variable::new("x", Position::dummy()),
                            vec![IfTypeBranch::new(
                                list_type.clone(),
                                None::new(Position::dummy())
                            )],
                            None,
                            Position::dummy()
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
                &type_context,
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_compiler::compile_concrete_list_name(&list_type, type_context.types())
                    .unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &type_context.list_type_configuration().list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_try_operation() {
        let type_context = TypeContext::dummy(Default::default(), Default::default());
        let list_type = types::List::new(types::None::new(Position::dummy()), Position::dummy());
        let union_type = types::Union::new(
            list_type.clone(),
            types::Record::new(
                &type_context.error_type_configuration().error_type_name,
                Position::dummy(),
            ),
            Position::dummy(),
        );

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![Definition::without_source(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", union_type)],
                        types::None::new(Position::dummy()),
                        TryOperation::new(
                            Some(list_type.clone().into()),
                            Variable::new("x", Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
                &type_context,
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_compiler::compile_concrete_list_name(&list_type, type_context.types())
                    .unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &type_context.list_type_configuration().list_type_name
                )
                .into()]),
            )])
        );
    }
}
