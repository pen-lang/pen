use super::{context::Context, type_, CompileError};
use crate::variant_type_collection;
use hir::{ir::*, types::Type};

pub fn compile(
    context: &Context,
    module: &Module,
) -> Result<Vec<mir::ir::TypeDefinition>, CompileError> {
    Ok(variant_type_collection::collect(context, module)?
        .into_iter()
        .map(|type_| compile_type_definition(context, &type_))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn compile_type_definition(
    context: &Context,
    type_: &Type,
) -> Result<Option<mir::ir::TypeDefinition>, CompileError> {
    Ok(match type_ {
        Type::Function(function_type) => Some(mir::ir::TypeDefinition::new(
            type_::compile_concrete_function_name(function_type, context.types())?,
            mir::types::RecordBody::new(vec![
                type_::compile_function(context, function_type)?.into()
            ]),
        )),
        Type::List(list_type) => Some(mir::ir::TypeDefinition::new(
            type_::compile_concrete_list_name(list_type, context.types())?,
            mir::types::RecordBody::new(vec![mir::types::Record::new(
                &context.configuration()?.list_type.list_type_name,
            )
            .into()]),
        )),
        Type::Map(map_type) => Some(mir::ir::TypeDefinition::new(
            type_::compile_concrete_map_name(map_type, context.types())?,
            mir::types::RecordBody::new(vec![mir::types::Record::new(
                &context.configuration()?.map_type.map_type_name,
            )
            .into()]),
        )),
        Type::Any(_)
        | Type::Boolean(_)
        | Type::Error(_)
        | Type::String(_)
        | Type::None(_)
        | Type::Number(_)
        | Type::Record(_) => None,
        Type::Reference(_) | Type::Union(_) => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::{
        test::{FunctionDefinitionFake, ModuleFake},
        types,
    };
    use position::{test::PositionFake, Position};

    #[test]
    fn compile_race_function() {
        let list_type = types::List::new(
            types::List::new(types::None::new(Position::fake()), Position::fake()),
            Position::fake(),
        );
        let context = Context::dummy(Default::default(), Default::default());

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        types::None::new(Position::fake()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![list_type.clone().into()],
                                    list_type.element().clone(),
                                    Position::fake()
                                )
                                .into()
                            ),
                            BuiltInFunction::new(BuiltInFunctionName::Race, Position::fake()),
                            vec![Variable::new("x", Position::fake()).into()],
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_list_name(
                    &types::List::new(types::Any::new(Position::fake()), Position::fake()),
                    context.types()
                )
                .unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &context.configuration().unwrap().list_type.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn compile_function_type_definition() {
        let function_type =
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            function_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let context = Context::dummy(Default::default(), Default::default());

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
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
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_function_name(&function_type, context.types()).unwrap(),
                mir::types::RecordBody::new(vec![type_::compile_function(
                    &context,
                    &function_type,
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
        let context = Context::dummy(Default::default(), Default::default());

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
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
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_list_name(&list_type, context.types()).unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &context.configuration().unwrap().list_type.list_type_name
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
        let context = Context::dummy(Default::default(), Default::default());
        let definition = FunctionDefinition::fake(
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
                &context,
                &Module::empty().set_function_definitions(vec![definition.clone(), definition]),
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_list_name(&list_type, context.types()).unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &context.configuration().unwrap().list_type.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_if_type() {
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let context = Context::dummy(Default::default(), Default::default());

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
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
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_list_name(&list_type, context.types()).unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &context.configuration().unwrap().list_type.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_equal_operation() {
        let context = Context::dummy(Default::default(), Default::default());
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            list_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("x", union_type)],
                        types::Boolean::new(Position::fake()),
                        EqualityOperation::new(
                            Some(list_type.clone().into()),
                            EqualityOperator::Equal,
                            Variable::new("x", Position::fake()),
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_list_name(&list_type, context.types()).unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &context.configuration().unwrap().list_type.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_try_operation() {
        let context = Context::dummy(Default::default(), Default::default());
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            list_type.clone(),
            types::Error::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
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
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_list_name(&list_type, context.types()).unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &context.configuration().unwrap().list_type.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_list_literal() {
        let context = Context::dummy(Default::default(), Default::default());
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            list_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
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
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_list_name(&list_type, context.types()).unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &context.configuration().unwrap().list_type.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_input_type_from_list_comprehension() {
        let context = Context::dummy(Default::default(), Default::default());
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            list_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "foo",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        ListComprehension::new(
                            types::None::new(Position::fake()),
                            None::new(Position::fake()),
                            vec![ListComprehensionBranch::new(
                                vec!["_".into()],
                                vec![ListComprehensionIteratee::new(
                                    Some(
                                        types::List::new(union_type.clone(), Position::fake())
                                            .into()
                                    ),
                                    List::new(union_type, vec![], Position::fake()),
                                )],
                                None,
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_list_name(&list_type, context.types()).unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &context.configuration().unwrap().list_type.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_output_type_from_list_comprehension() {
        let context = Context::dummy(Default::default(), Default::default());
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let union_type = types::Union::new(
            list_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "foo",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        ListComprehension::new(
                            union_type,
                            None::new(Position::fake()),
                            vec![ListComprehensionBranch::new(
                                vec!["_".into()],
                                vec![ListComprehensionIteratee::new(
                                    Some(list_type.clone().into()),
                                    List::new(
                                        types::None::new(Position::fake()),
                                        vec![],
                                        Position::fake()
                                    ),
                                )],
                                None,
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_list_name(&list_type, context.types()).unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &context.configuration().unwrap().list_type.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_if_list() {
        let context = Context::dummy(Default::default(), Default::default());
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "foo",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        IfList::new(
                            Some(list_type.clone().into()),
                            List::new(list_type.element().clone(), vec![], Position::fake()),
                            "x",
                            "xs",
                            None::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            ),
            Ok(vec![mir::ir::TypeDefinition::new(
                type_::compile_concrete_list_name(&list_type, context.types()).unwrap(),
                mir::types::RecordBody::new(vec![mir::types::Record::new(
                    &context.configuration().unwrap().list_type.list_type_name
                )
                .into()]),
            )])
        );
    }

    #[test]
    fn collect_type_from_if_map() {
        let context = Context::dummy(Default::default(), Default::default());
        let key_type = types::List::new(types::Number::new(Position::fake()), Position::fake());
        let value_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let map_type = types::Map::new(key_type.clone(), value_type.clone(), Position::fake());

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "foo",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        IfMap::new(
                            Some(map_type.key().clone()),
                            Some(map_type.value().clone()),
                            "x",
                            Map::new(
                                map_type.key().clone(),
                                map_type.value().clone(),
                                vec![],
                                Position::fake()
                            ),
                            Number::new(42.0, Position::fake()),
                            None::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            ),
            Ok(vec![
                mir::ir::TypeDefinition::new(
                    type_::compile_concrete_list_name(&key_type, context.types()).unwrap(),
                    mir::types::RecordBody::new(vec![mir::types::Record::new(
                        &context.configuration().unwrap().list_type.list_type_name
                    )
                    .into()]),
                ),
                mir::ir::TypeDefinition::new(
                    type_::compile_concrete_list_name(&value_type, context.types()).unwrap(),
                    mir::types::RecordBody::new(vec![mir::types::Record::new(
                        &context.configuration().unwrap().list_type.list_type_name
                    )
                    .into()]),
                )
            ])
        );
    }

    #[test]
    fn collect_type_from_map_literal() {
        let context = Context::dummy(Default::default(), Default::default());
        let key_type = types::List::new(types::Number::new(Position::fake()), Position::fake());
        let value_type = types::List::new(types::None::new(Position::fake()), Position::fake());

        assert_eq!(
            compile(
                &context,
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "foo",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Map::new(
                            key_type.clone(),
                            value_type.clone(),
                            vec![],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            ),
            Ok(vec![
                mir::ir::TypeDefinition::new(
                    type_::compile_concrete_list_name(&key_type, context.types()).unwrap(),
                    mir::types::RecordBody::new(vec![mir::types::Record::new(
                        &context.configuration().unwrap().list_type.list_type_name
                    )
                    .into()]),
                ),
                mir::ir::TypeDefinition::new(
                    type_::compile_concrete_list_name(&value_type, context.types()).unwrap(),
                    mir::types::RecordBody::new(vec![mir::types::Record::new(
                        &context.configuration().unwrap().list_type.list_type_name
                    )
                    .into()]),
                )
            ])
        );
    }
}
