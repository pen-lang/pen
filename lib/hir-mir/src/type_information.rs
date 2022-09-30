pub mod debug;

use crate::{context::CompileContext, generic_type_collection, type_, CompileError};
use fnv::FnvHashSet;
use hir::{
    analysis::{type_canonicalizer, type_collector},
    ir::*,
    types::{self, Type},
};
use itertools::Itertools;

pub const DEBUG_FUNCTION_INDEX: usize = 0;

pub fn compile(
    context: &CompileContext,
    module: &Module,
) -> Result<mir::ir::TypeInformation, CompileError> {
    Ok(mir::ir::TypeInformation::new(
        vec![debug::compile_function_type()],
        collect_types(context, module)?
            .iter()
            .map(|type_| {
                Ok((
                    type_::compile_concrete(context, type_)?,
                    vec![debug::compile_function_name(context, type_)?],
                ))
            })
            .collect::<Result<_, CompileError>>()?,
    ))
}

pub fn compile_functions(
    context: &CompileContext,
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
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unique_by(|declaration| declaration.name().to_owned())
            .collect(),
        internal_types
            .iter()
            .map(|type_| debug::compile_function_definition(context, type_))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .unique_by(|definition| definition.name().to_owned())
            .map(|definition| mir::ir::GlobalFunctionDefinition::new(definition, false))
            .collect(),
    ))
}

fn collect_types(
    context: &CompileContext,
    module: &Module,
) -> Result<FnvHashSet<Type>, CompileError> {
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

    fn create_context(module: &Module) -> CompileContext {
        CompileContext::new(module, Some(COMPILE_CONFIGURATION.clone()))
    }

    fn create_default_type_information(
        context: &CompileContext,
    ) -> FnvHashMap<mir::types::Type, Vec<String>> {
        [
            (
                mir::types::Type::Boolean,
                vec![debug::compile_function_name(
                    context,
                    &types::Boolean::new(Position::fake()).into(),
                )
                .unwrap()],
            ),
            (
                mir::types::Type::ByteString,
                vec![debug::compile_function_name(
                    context,
                    &types::ByteString::new(Position::fake()).into(),
                )
                .unwrap()],
            ),
            (
                mir::types::Type::None,
                vec![debug::compile_function_name(
                    context,
                    &types::None::new(Position::fake()).into(),
                )
                .unwrap()],
            ),
            (
                mir::types::Type::Number,
                vec![debug::compile_function_name(
                    context,
                    &types::Number::new(Position::fake()).into(),
                )
                .unwrap()],
            ),
            (
                mir::types::Record::new("error").into(),
                vec![debug::compile_function_name(
                    context,
                    &types::Error::new(Position::fake()).into(),
                )
                .unwrap()],
            ),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn compile_nothing() {
        let module = Module::empty();
        let context = create_context(&module);

        assert_eq!(
            compile(&context, &module).unwrap(),
            mir::ir::TypeInformation::new(
                vec![debug::compile_function_type()],
                create_default_type_information(&context)
            )
        )
    }

    #[test]
    fn compile_without_compile_configuration() {
        let module = Module::empty();
        let context = CompileContext::new(&module, None);

        assert_eq!(
            compile(&context, &module).unwrap(),
            mir::ir::TypeInformation::new(
                vec![debug::compile_function_type()],
                [
                    (
                        mir::types::Type::Boolean,
                        vec![debug::compile_function_name(
                            &context,
                            &types::Boolean::new(Position::fake()).into()
                        )
                        .unwrap()]
                    ),
                    (
                        mir::types::Type::ByteString,
                        vec![debug::compile_function_name(
                            &context,
                            &types::ByteString::new(Position::fake()).into()
                        )
                        .unwrap()]
                    ),
                    (
                        mir::types::Type::None,
                        vec![debug::compile_function_name(
                            &context,
                            &types::None::new(Position::fake()).into()
                        )
                        .unwrap()]
                    ),
                    (
                        mir::types::Type::Number,
                        vec![debug::compile_function_name(
                            &context,
                            &types::Number::new(Position::fake()).into()
                        )
                        .unwrap()]
                    ),
                ]
                .into_iter()
                .collect()
            )
        )
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
            mir::ir::TypeInformation::new(
                vec![debug::compile_function_type()],
                create_default_type_information(&context)
            )
        )
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
            mir::ir::TypeInformation::new(
                vec![debug::compile_function_type()],
                create_default_type_information(&context)
            )
        )
    }

    #[test]
    fn compile_function() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                List::new(
                    types::Function::new(
                        vec![],
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
            compile(&context, &module).unwrap().information().len(),
            create_default_type_information(&context).len() + 1
        )
    }

    #[test]
    fn compile_list() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                List::new(
                    types::List::new(types::Any::new(Position::fake()), Position::fake()),
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
        )
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
        )
    }

    #[test]
    fn compile_map() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                List::new(
                    types::Map::new(
                        types::None::new(Position::fake()),
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
            compile(&context, &module).unwrap().information().len(),
            create_default_type_information(&context).len() + 1
        )
    }
}
