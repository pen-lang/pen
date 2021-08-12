use super::{type_compiler, type_context::TypeContext, CompileError};
use crate::{
    hir::{analysis::expression_visitor, *},
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
    Ok(collect_types(module, type_context.types())?
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

fn collect_types(
    module: &Module,
    types: &HashMap<String, Type>,
) -> Result<HashSet<Type>, TypeError> {
    let mut lower_types = HashSet::new();

    expression_visitor::visit(module, |expression| match expression {
        Expression::IfType(if_) => {
            lower_types.extend(
                if_.branches()
                    .iter()
                    .map(|branch| branch.type_())
                    .chain(if_.else_().map(|branch| branch.type_()).flatten())
                    .cloned(),
            );
        }
        Expression::TypeCoercion(coercion) => {
            lower_types.insert(coercion.from().clone());
        }
        Expression::Operation(operation) => match operation {
            Operation::Try(operation) => {
                lower_types.extend(operation.type_().cloned());
            }
            _ => {}
        },
        _ => {}
    });

    lower_types
        .into_iter()
        .map(|type_| type_canonicalizer::canonicalize(&type_, types))
        .collect()
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
