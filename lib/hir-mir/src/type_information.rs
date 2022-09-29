pub mod debug;

use crate::{context::CompileContext, generic_type_collection, type_, CompileError};
use fnv::FnvHashSet;
use hir::{
    analysis::type_collector,
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
        vec![debug::compile_function_type().into()],
        collect_types(context, module)?
            .iter()
            .map(|type_| {
                Ok((
                    type_::compile(context, type_)?,
                    vec![debug::compile_function_name(context, type_)?],
                ))
            })
            .collect::<Result<_, CompileError>>()?,
    ))
}

// TODO Compile function declarations for external records.
pub fn compile_function_definitions(
    context: &CompileContext,
    module: &Module,
) -> Result<Vec<mir::ir::GlobalFunctionDefinition>, CompileError> {
    Ok(collect_types(context, module)?
        .iter()
        .map(|type_| debug::compile_function_definition(context, type_))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .unique_by(|definition| definition.name().to_owned())
        .map(|definition| mir::ir::GlobalFunctionDefinition::new(definition, false))
        .collect())
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
    .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::test::ModuleFake;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn create_context(module: &Module) -> CompileContext {
        CompileContext::new(module, Some(COMPILE_CONFIGURATION.clone()))
    }

    #[test]
    fn compile_nothing() {
        let module = Module::empty();
        let context = create_context(&module);

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
                    (
                        mir::types::Record::new("error").into(),
                        vec![debug::compile_function_name(
                            &context,
                            &types::Error::new(Position::fake()).into()
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
}
