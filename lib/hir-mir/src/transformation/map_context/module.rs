use super::context_function_name;
use crate::{
    CompileError,
    context::Context,
    transformation::{collection_type, equal_operation, hash_calculation},
};
use fnv::FnvHashSet;
use hir::{
    analysis::{
        AnalysisError, expression_visitor, type_canonicalizer, type_comparability_checker,
        type_visitor,
    },
    ir::*,
    types,
    types::Type,
};
use position::Position;

pub fn transform(context: &Context, module: &Module) -> Result<Module, CompileError> {
    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .cloned()
            .chain(
                collect_map_types(context, module)?
                    .into_iter()
                    .map(|type_| transform_map_context_function_definition(context, &type_))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .collect(),
        module.position().clone(),
    ))
}

fn transform_map_context_function_definition(
    context: &Context,
    map_type: &types::Map,
) -> Result<FunctionDefinition, CompileError> {
    let position = map_type.position();
    let context_type = collection_type::transform_map_context(context, position)?;
    let name = context_function_name(map_type, context.types())?;

    Ok(FunctionDefinition::new(
        &name,
        &name,
        Lambda::new(
            vec![],
            context_type.clone(),
            Call::new(
                Some(compile_context_function_type(&context_type, position).into()),
                Variable::new(
                    &context.configuration()?.map_type.context_function_name,
                    position.clone(),
                ),
                [
                    equal_operation::function::transform(context, map_type.key())?,
                    hash_calculation::function::transform(context, map_type.key())?,
                ]
                .into_iter()
                .chain(
                    if type_comparability_checker::check(
                        map_type.value(),
                        context.types(),
                        context.records(),
                    )? {
                        [
                            equal_operation::function::transform(context, map_type.value())?,
                            hash_calculation::function::transform(context, map_type.value())?,
                        ]
                    } else {
                        [
                            compile_fake_equal_function(position).into(),
                            compile_fake_hash_function(position).into(),
                        ]
                    },
                )
                .collect(),
                position.clone(),
            ),
            position.clone(),
        ),
        None,
        false,
        position.clone(),
    ))
}

fn compile_context_function_type(context_type: &Type, position: &Position) -> types::Function {
    let any_type = Type::from(types::Any::new(position.clone()));
    let equal_function_type = Type::from(types::Function::new(
        vec![any_type.clone(), any_type.clone()],
        types::Boolean::new(position.clone()),
        position.clone(),
    ));
    let hash_function_type = Type::from(types::Function::new(
        vec![any_type],
        types::Number::new(position.clone()),
        position.clone(),
    ));

    types::Function::new(
        vec![
            equal_function_type.clone(),
            hash_function_type.clone(),
            equal_function_type,
            hash_function_type,
        ],
        context_type.clone(),
        position.clone(),
    )
}

fn compile_fake_equal_function(position: &Position) -> Lambda {
    Lambda::new(
        vec![
            Argument::new("$value1", types::Any::new(position.clone())),
            Argument::new("$value2", types::Any::new(position.clone())),
        ],
        types::Boolean::new(position.clone()),
        Boolean::new(false, position.clone()),
        position.clone(),
    )
}

fn compile_fake_hash_function(position: &Position) -> Lambda {
    Lambda::new(
        vec![Argument::new("$value", types::Any::new(position.clone()))],
        types::Number::new(position.clone()),
        Number::new(0.0, position.clone()),
        position.clone(),
    )
}

fn collect_map_types(
    context: &Context,
    module: &Module,
) -> Result<FnvHashSet<types::Map>, AnalysisError> {
    let mut map_types = FnvHashSet::default();

    type_visitor::visit(module, |type_| {
        if let Type::Map(map_type) = type_ {
            map_types.insert(map_type.clone());
        }
    });

    expression_visitor::visit(module, |expression| match expression {
        Expression::IfMap(if_) => {
            map_types.insert(types::Map::new(
                if_.key_type().unwrap().clone(),
                if_.value_type().unwrap().clone(),
                if_.position().clone(),
            ));
        }
        Expression::Map(map) => {
            map_types.insert(types::Map::new(
                map.key_type().clone(),
                map.value_type().clone(),
                map.position().clone(),
            ));
        }
        _ => {}
    });

    map_types
        .into_iter()
        .map(|type_| {
            Ok(types::Map::new(
                type_canonicalizer::canonicalize(type_.key(), context.types())?,
                type_canonicalizer::canonicalize(type_.value(), context.types())?,
                type_.position().clone(),
            ))
        })
        .collect::<Result<_, _>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::test::{FunctionDefinitionFake, ModuleFake};
    use position::test::PositionFake;

    #[test]
    fn transform_none_key_and_none_value() {
        let context = Context::dummy(Default::default(), Default::default());

        insta::assert_debug_snapshot!(transform(
            &context,
            &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                Lambda::new(
                    vec![],
                    collection_type::transform_map_context(&context, &Position::fake()).unwrap(),
                    Map::new(
                        types::None::new(Position::fake()),
                        types::None::new(Position::fake()),
                        vec![],
                        Position::fake()
                    ),
                    Position::fake()
                ),
                false
            )])
        ));
    }

    #[test]
    fn transform_function_value() {
        let context = Context::dummy(Default::default(), Default::default());

        insta::assert_debug_snapshot!(transform(
            &context,
            &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                Lambda::new(
                    vec![],
                    collection_type::transform_map_context(&context, &Position::fake()).unwrap(),
                    Map::new(
                        types::None::new(Position::fake()),
                        types::Function::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Position::fake()
                        ),
                        vec![],
                        Position::fake()
                    ),
                    Position::fake()
                ),
                false
            )])
        ));
    }

    #[test]
    fn do_not_create_duplicate_map_contexts() {
        let context = Context::dummy(
            [
                ("foo".into(), types::None::new(Position::fake()).into()),
                ("bar".into(), types::None::new(Position::fake()).into()),
            ]
            .into_iter()
            .collect(),
            Default::default(),
        );
        let module = Module::empty().set_function_definitions(vec![
            FunctionDefinition::fake(
                "f",
                Lambda::new(
                    vec![],
                    collection_type::transform_map_context(&context, &Position::fake()).unwrap(),
                    Map::new(
                        types::None::new(Position::fake()),
                        types::None::new(Position::fake()),
                        vec![],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            ),
            FunctionDefinition::fake(
                "f",
                Lambda::new(
                    vec![],
                    collection_type::transform_map_context(&context, &Position::fake()).unwrap(),
                    Map::new(
                        types::Reference::new("foo", Position::fake()),
                        types::Reference::new("bar", Position::fake()),
                        vec![],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            ),
        ]);

        assert_eq!(
            transform(&context, &module)
                .unwrap()
                .function_definitions()
                .len()
                - module.function_definitions().len(),
            1,
        );
    }
}
