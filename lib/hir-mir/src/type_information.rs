pub mod debug;

use crate::{context::Context, generic_type_collection, type_, CompileError};
use fnv::FnvHashSet;
use hir::{
    analysis::{type_canonicalizer, type_collector},
    ir::*,
    types::{self, Type},
};

pub fn compile(
    context: &Context,
    module: &Module,
) -> Result<mir::ir::TypeInformation, CompileError> {
    Ok(mir::ir::TypeInformation::new(
        collect_types(context, module)?
            .iter()
            .map(|type_| {
                Ok((
                    type_::compile_concrete(context, type_)?,
                    debug::compile_function_name(context, type_)?,
                ))
            })
            .collect::<Result<_, CompileError>>()?,
        debug::compile_default_function_name().into(),
    ))
}

pub fn compile_functions(
    context: &Context,
    module: &Module,
) -> Result<
    (
        Vec<mir::ir::FunctionDeclaration>,
        Vec<mir::ir::GlobalFunctionDefinition>,
    ),
    CompileError,
> {
    let types = collect_types(context, module)?;
    let external_record_names = module
        .type_definitions()
        .iter()
        .filter(|definition| definition.is_external())
        .map(|definition| definition.name())
        .collect::<FnvHashSet<_>>();
    let internal_record_names = module
        .type_definitions()
        .iter()
        .filter(|definition| !definition.is_external())
        .map(|definition| definition.name())
        .collect::<FnvHashSet<_>>();
    let (external_types, internal_types) =
        types
            .iter()
            .partition::<FnvHashSet<_>, _>(|type_| match type_ {
                Type::Record(type_) => external_record_names.contains(type_.name()),
                _ => false,
            });

    Ok((
        external_types
            .iter()
            .map(|type_| debug::compile_function_declaration(context, type_))
            .collect::<Result<Vec<_>, _>>()?,
        internal_types
            .iter()
            .map(|type_| -> Result<_, CompileError> {
                Ok(
                    debug::compile_function_definition(context, type_)?.map(|definition| {
                        mir::ir::GlobalFunctionDefinition::new(
                            definition,
                            match type_ {
                                Type::Record(record) => {
                                    internal_record_names.contains(record.name())
                                }
                                _ => false,
                            },
                        )
                    }),
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .chain([mir::ir::GlobalFunctionDefinition::new(
                debug::compile_default_function_definition(),
                false,
            )])
            .collect(),
    ))
}

fn collect_types(context: &Context, module: &Module) -> Result<FnvHashSet<Type>, CompileError> {
    let position = module.position();

    Ok([
        types::Boolean::new(position.clone()).into(),
        types::ByteString::new(position.clone()).into(),
        types::None::new(position.clone()).into(),
        types::Number::new(position.clone()).into(),
    ]
    .into_iter()
    .chain(
        context
            .configuration()
            .map(|_| types::Error::new(position.clone()).into()),
    )
    .chain(generic_type_collection::collect(context, module)?)
    .chain(
        type_collector::collect_records(module)
            .into_values()
            .map(Type::from),
    )
    .map(|type_| type_canonicalizer::canonicalize(&type_, context.types()))
    .collect::<Result<Vec<_>, _>>()?
    .into_iter()
    .filter(|type_| !matches!(&type_, Type::Any(_) | Type::Union(_)))
    .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use fnv::FnvHashMap;
    use hir::test::{FunctionDefinitionFake, ModuleFake};
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn create_context(module: &Module) -> Context {
        Context::new(module, Some(COMPILE_CONFIGURATION.clone()))
    }

    fn compile_type_information(
        information: FnvHashMap<mir::types::Type, String>,
    ) -> mir::ir::TypeInformation {
        mir::ir::TypeInformation::new(information, debug::compile_default_function_name().into())
    }

    fn create_default_type_information(context: &Context) -> FnvHashMap<mir::types::Type, String> {
        [
            (
                mir::types::Type::Boolean,
                debug::compile_function_name(
                    context,
                    &types::Boolean::new(Position::fake()).into(),
                )
                .unwrap(),
            ),
            (
                mir::types::Type::ByteString,
                debug::compile_function_name(
                    context,
                    &types::ByteString::new(Position::fake()).into(),
                )
                .unwrap(),
            ),
            (
                mir::types::Type::None,
                debug::compile_function_name(context, &types::None::new(Position::fake()).into())
                    .unwrap(),
            ),
            (
                mir::types::Type::Number,
                debug::compile_function_name(context, &types::Number::new(Position::fake()).into())
                    .unwrap(),
            ),
            (
                mir::types::Record::new("error").into(),
                debug::compile_function_name(context, &types::Error::new(Position::fake()).into())
                    .unwrap(),
            ),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn compile_empty() {
        let module = Module::empty();
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap(),
            compile_type_information(create_default_type_information(&context),)
        );

        for type_ in &[
            types::Boolean::new(Position::fake()).into(),
            types::ByteString::new(Position::fake()).into(),
            types::Error::new(Position::fake()).into(),
            types::None::new(Position::fake()).into(),
            types::Number::new(Position::fake()).into(),
        ] {
            assert!(!compile_functions(&context, &module)
                .unwrap()
                .1
                .iter()
                .find(|definition| definition.definition().name()
                    == debug::compile_function_name(&context, type_).unwrap())
                .unwrap()
                .is_public());
        }
    }

    #[test]
    fn compile_without_compile_configuration() {
        let module = Module::empty();
        let context = Context::new(&module, None);

        assert_eq!(
            compile(&context, &module).unwrap(),
            compile_type_information(
                [
                    (
                        mir::types::Type::Boolean,
                        debug::compile_function_name(
                            &context,
                            &types::Boolean::new(Position::fake()).into()
                        )
                        .unwrap()
                    ),
                    (
                        mir::types::Type::ByteString,
                        debug::compile_function_name(
                            &context,
                            &types::ByteString::new(Position::fake()).into()
                        )
                        .unwrap()
                    ),
                    (
                        mir::types::Type::None,
                        debug::compile_function_name(
                            &context,
                            &types::None::new(Position::fake()).into()
                        )
                        .unwrap()
                    ),
                    (
                        mir::types::Type::Number,
                        debug::compile_function_name(
                            &context,
                            &types::Number::new(Position::fake()).into()
                        )
                        .unwrap()
                    ),
                ]
                .into_iter()
                .collect(),
            )
        );
    }

    #[test]
    fn compile_any() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                List::new(types::Any::new(Position::fake()), vec![], Position::fake()),
                Position::fake(),
            ),
            false,
        )]);
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap(),
            compile_type_information(create_default_type_information(&context),)
        );
    }

    #[test]
    fn compile_union() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                List::new(
                    types::Union::new(
                        types::Number::new(Position::fake()),
                        types::None::new(Position::fake()),
                        Position::fake(),
                    ),
                    vec![],
                    Position::fake(),
                ),
                Position::fake(),
            ),
            false,
        )]);
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap(),
            compile_type_information(create_default_type_information(&context))
        )
    }

    #[test]
    fn compile_function() {
        let function_type =
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                List::new(function_type.clone(), vec![], Position::fake()),
                Position::fake(),
            ),
            false,
        )]);
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap().information().len(),
            create_default_type_information(&context).len() + 1
        );
        assert!(!compile_functions(&context, &module)
            .unwrap()
            .1
            .iter()
            .find(|definition| definition.definition().name()
                == debug::compile_function_name(&context, &function_type.clone().into()).unwrap())
            .unwrap()
            .is_public());
    }

    #[test]
    fn compile_list() {
        let list_type = types::List::new(types::Any::new(Position::fake()), Position::fake());
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                List::new(list_type.clone(), vec![], Position::fake()),
                Position::fake(),
            ),
            false,
        )]);
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap().information().len(),
            create_default_type_information(&context).len() + 1
        );
        assert!(!compile_functions(&context, &module)
            .unwrap()
            .1
            .iter()
            .find(|definition| definition.definition().name()
                == debug::compile_function_name(&context, &list_type.clone().into()).unwrap())
            .unwrap()
            .is_public());
    }

    #[test]
    fn compile_two_lists() {
        let module = Module::empty().set_function_definitions(vec![
            FunctionDefinition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    List::new(
                        types::List::new(types::None::new(Position::fake()), Position::fake()),
                        vec![],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            ),
            FunctionDefinition::fake(
                "g",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    List::new(
                        types::List::new(types::Number::new(Position::fake()), Position::fake()),
                        vec![],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            ),
        ]);
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap().information().len(),
            create_default_type_information(&context).len() + 2
        );
    }

    #[test]
    fn compile_map() {
        let map_type = types::Map::new(
            types::None::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                List::new(map_type.clone(), vec![], Position::fake()),
                Position::fake(),
            ),
            false,
        )]);
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap().information().len(),
            create_default_type_information(&context).len() + 1
        );
        assert!(!compile_functions(&context, &module)
            .unwrap()
            .1
            .iter()
            .find(|definition| definition.definition().name()
                == debug::compile_function_name(&context, &map_type.clone().into()).unwrap())
            .unwrap()
            .is_public());
    }

    #[test]
    fn compile_external_record() {
        let module = Module::empty()
            .set_type_definitions(vec![TypeDefinition::new(
                "r",
                "r",
                vec![],
                false,
                false,
                true,
                Position::fake(),
            )])
            .set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    List::new(
                        types::Record::new("r", Position::fake()),
                        vec![],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )]);
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap().information().len(),
            create_default_type_information(&context).len() + 1
        );
        assert_eq!(
            compile_functions(&context, &module)
                .unwrap()
                .0
                .iter()
                .filter(|declaration| declaration.name()
                    == debug::compile_function_name(
                        &context,
                        &types::Record::new("r", Position::fake()).into()
                    )
                    .unwrap())
                .count(),
            1
        );
    }

    #[test]
    fn compile_internal_record() {
        let module = Module::empty()
            .set_type_definitions(vec![TypeDefinition::new(
                "r",
                "r",
                vec![],
                false,
                false,
                false,
                Position::fake(),
            )])
            .set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    List::new(
                        types::Record::new("r", Position::fake()),
                        vec![],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )]);
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap().information().len(),
            create_default_type_information(&context).len() + 1
        );
        assert!(compile_functions(&context, &module)
            .unwrap()
            .1
            .iter()
            .find(|definition| definition.definition().name()
                == debug::compile_function_name(
                    &context,
                    &types::Record::new("r", Position::fake()).into()
                )
                .unwrap())
            .unwrap()
            .is_public());
    }

    #[test]
    fn compile_record_field() {
        let field_type =
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());

        let module = Module::empty()
            .set_type_definitions(vec![TypeDefinition::new(
                "r",
                "r",
                vec![types::RecordField::new("x", field_type.clone())],
                false,
                false,
                false,
                Position::fake(),
            )])
            .set_function_definitions(vec![FunctionDefinition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    List::new(
                        types::Record::new("r", Position::fake()),
                        vec![],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )]);
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap().information().len(),
            create_default_type_information(&context).len() + 2
        );
        assert!(!compile_functions(&context, &module)
            .unwrap()
            .1
            .iter()
            .find(|definition| definition.definition().name()
                == debug::compile_function_name(&context, &field_type.clone().into()).unwrap())
            .unwrap()
            .is_public());
    }
}
