use super::{compile_context::CompileContext, type_compiler, CompileError};
use hir::{
    analysis::{
        ir::expression_visitor,
        types::{union_type_member_calculator, TypeError},
    },
    ir::*,
    types::Type,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn compile(
    module: &Module,
    compile_context: &CompileContext,
) -> Result<Vec<mir::ir::TypeDefinition>, CompileError> {
    Ok(collect_types(module, compile_context.types())?
        .into_iter()
        .map(|type_| compile_type_definition(&type_, compile_context))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn compile_type_definition(
    type_: &Type,
    compile_context: &CompileContext,
) -> Result<Option<mir::ir::TypeDefinition>, CompileError> {
    Ok(match type_ {
        Type::Function(function_type) => Some(mir::ir::TypeDefinition::new(
            type_compiler::compile_concrete_function_name(function_type, compile_context.types())?,
            mir::types::RecordBody::new(vec![type_compiler::compile_function(
                function_type,
                compile_context,
            )?
            .into()]),
        )),
        Type::List(list_type) => Some(mir::ir::TypeDefinition::new(
            type_compiler::compile_concrete_list_name(list_type, compile_context.types())?,
            mir::types::RecordBody::new(vec![mir::types::Record::new(
                &compile_context.compile_configuration().list_type_configuration.list_type_name,
            )
            .into()]),
        )),
        _ => None,
    })
}

fn collect_types(
    module: &Module,
    types: &BTreeMap<String, Type>,
) -> Result<BTreeSet<Type>, TypeError> {
    let mut lower_types = BTreeSet::new();

    expression_visitor::visit(module, |expression| match expression {
        Expression::IfType(if_) => {
            lower_types.extend(
                if_.branches()
                    .iter()
                    .map(|branch| branch.type_())
                    .chain(if_.else_().and_then(|branch| branch.type_()))
                    .cloned(),
            );
        }
        Expression::List(list) => {
            lower_types.insert(list.type_().clone());
        }
        Expression::TypeCoercion(coercion) => {
            lower_types.insert(coercion.from().clone());
        }
        Expression::Operation(Operation::Try(operation)) => {
            lower_types.extend(operation.type_().cloned());
        }
        _ => {}
    });

    Ok(lower_types
        .into_iter()
        .map(|type_| union_type_member_calculator::calculate(&type_, types))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::{
        test::{DefinitionFake, ModuleFake},
        types,
    };
    use position::{test::PositionFake, Position};

    #[test]
    fn compile_function_type_definition() {
        let function_type =
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            function_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let compile_context = CompileContext::dummy(Default::default(), Default::default());

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", function_type.clone())],
                        types::None::new(Position::fake()),
                        TypeCoercion::new(
                            function_type.clone(),
                            union_type,
                            Variable::new("x", Position::fake()),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
                &compile_context,
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_compiler::compile_concrete_function_name(
                    &function_type,
                    compile_context.types()
                )
                .unwrap(),
                mir::types::RecordBody::new(vec![type_compiler::compile_function(
                    &function_type,
                    &compile_context
                )
                .unwrap()
                .into()]),
            )])
        );
    }

    #[test]
    fn compile_list_type_definition() {
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            list_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let compile_context = CompileContext::dummy(Default::default(), Default::default());

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        types::None::new(Position::fake()),
                        TypeCoercion::new(
                            list_type.clone(),
                            union_type,
                            Variable::new("x", Position::fake()),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
                &compile_context,
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_compiler::compile_concrete_list_name(&list_type, compile_context.types())
                    .unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &compile_context.compile_configuration().list_type_configuration.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn compile_duplicate_list_type_definitions() {
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            list_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let compile_context = CompileContext::dummy(Default::default(), Default::default());
        let definition = Definition::fake(
            "foo",
            Lambda::new(
                vec![Argument::new("x", list_type.clone())],
                types::None::new(Position::fake()),
                TypeCoercion::new(
                    list_type.clone(),
                    union_type,
                    Variable::new("x", Position::fake()),
                    Position::fake(),
                ),
                Position::fake(),
            ),
            false,
        );

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![definition.clone(), definition]),
                &compile_context,
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_compiler::compile_concrete_list_name(&list_type, compile_context.types())
                    .unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &compile_context.compile_configuration().list_type_configuration.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_if_type() {
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let compile_context = CompileContext::dummy(Default::default(), Default::default());

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "x",
                            Variable::new("x", Position::fake()),
                            vec![IfTypeBranch::new(
                                list_type.clone(),
                                None::new(Position::fake())
                            )],
                            None,
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
                &compile_context,
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_compiler::compile_concrete_list_name(&list_type, compile_context.types())
                    .unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &compile_context.compile_configuration().list_type_configuration.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_try_operation() {
        let compile_context = CompileContext::dummy(Default::default(), Default::default());
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            list_type.clone(),
            types::Record::new(
                &compile_context.compile_configuration().error_type_configuration.error_type_name,
                Position::fake(),
            ),
            Position::fake(),
        );

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", union_type)],
                        types::None::new(Position::fake()),
                        TryOperation::new(
                            Some(list_type.clone().into()),
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
                &compile_context,
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_compiler::compile_concrete_list_name(&list_type, compile_context.types())
                    .unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &compile_context.compile_configuration().list_type_configuration.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_list_literal() {
        let compile_context = CompileContext::dummy(Default::default(), Default::default());
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            list_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        types::None::new(Position::fake()),
                        List::new(
                            union_type,
                            vec![ListElement::Single(
                                Variable::new("x", Position::fake()).into()
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
                &compile_context,
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_compiler::compile_concrete_list_name(&list_type, compile_context.types())
                    .unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &compile_context.compile_configuration().list_type_configuration.list_type_name
                )
                .into()]),
            )])
        );
    }
}
