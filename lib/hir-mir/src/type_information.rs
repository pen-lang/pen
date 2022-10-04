pub mod debug;

use crate::{context::Context, generic_type_collection, type_, CompileError};
use fnv::FnvHashSet;
use hir::{
    analysis::{type_canonicalizer, type_collector, type_id_calculator},
    ir::*,
    types::{self, Type},
};

const DEFAULT_TYPE_INFORMATION_FUNCTION_NAME: &str = "hir:type_information:default";
const TYPE_INFORMATION_RECORD_NAME: &str = "hir:type_information:record";

pub fn compile_type_information(
    context: &Context,
    module: &Module,
) -> Result<mir::ir::TypeInformation, CompileError> {
    Ok(mir::ir::TypeInformation::new(
        collect_types(context, module)?
            .iter()
            .map(|type_| {
                Ok((
                    type_::compile_concrete(context, type_)?,
                    compile_function_name(context, type_)?,
                ))
            })
            .collect::<Result<_, CompileError>>()?,
        DEFAULT_TYPE_INFORMATION_FUNCTION_NAME.into(),
    ))
}

fn compile_function(argument: impl Into<mir::ir::Expression>, index: usize) -> mir::ir::Expression {
    let argument = argument.into();

    mir::ir::RecordField::new(
        compile_type_information_type(),
        index,
        mir::ir::Call::new(
            compile_function_type(),
            mir::ir::TypeInformationFunction::new(argument),
            vec![],
        ),
    )
    .into()
}

pub fn compile_type_information_type_definition() -> mir::ir::TypeDefinition {
    mir::ir::TypeDefinition::new(
        TYPE_INFORMATION_RECORD_NAME,
        mir::types::RecordBody::new(vec![debug::compile_function_type().into()]),
    )
}

pub fn compile_function_declarations_and_definitions(
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
            .map(|type_| compile_function_declarations(context, type_))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect(),
        internal_types
            .iter()
            .map(|type_| -> Result<_, CompileError> {
                compile_function_definitions(
                    context,
                    type_,
                    match type_ {
                        Type::Record(record) => internal_record_names.contains(record.name()),
                        _ => false,
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .chain(compile_default_function_definitions())
            .collect(),
    ))
}

fn compile_function_name(context: &Context, type_: &Type) -> Result<String, CompileError> {
    Ok(format!(
        "hir:type_information:{}",
        type_id_calculator::calculate(type_, context.types())?
    ))
}

fn compile_function_type() -> mir::types::Function {
    mir::types::Function::new(vec![], compile_type_information_type())
}

fn compile_function_declarations(
    context: &Context,
    type_: &Type,
) -> Result<Vec<mir::ir::FunctionDeclaration>, CompileError> {
    Ok(vec![
        debug::compile_function_declaration(context, type_)?,
        mir::ir::FunctionDeclaration::new(
            compile_function_name(context, type_)?,
            compile_function_type(),
        ),
    ])
}

fn compile_function_definitions(
    context: &Context,
    type_: &Type,
    public: bool,
) -> Result<Vec<mir::ir::GlobalFunctionDefinition>, CompileError> {
    let type_information_type = compile_type_information_type();

    Ok([
        debug::compile_function_definition(context, type_)?,
        mir::ir::FunctionDefinition::thunk(
            compile_function_name(context, type_)?,
            type_information_type.clone(),
            mir::ir::Record::new(
                type_information_type,
                vec![mir::ir::Variable::new(debug::compile_function_name(context, type_)?).into()],
            ),
        ),
    ]
    .into_iter()
    .map(|definition| mir::ir::GlobalFunctionDefinition::new(definition, public))
    .collect())
}

fn compile_default_function_definitions() -> Vec<mir::ir::GlobalFunctionDefinition> {
    let type_information_type = compile_type_information_type();

    [
        debug::compile_default_function_definition(),
        mir::ir::FunctionDefinition::new(
            DEFAULT_TYPE_INFORMATION_FUNCTION_NAME,
            vec![],
            type_information_type.clone(),
            mir::ir::Record::new(
                type_information_type,
                vec![mir::ir::Variable::new(debug::compile_default_function_name()).into()],
            ),
        ),
    ]
    .into_iter()
    .map(|definition| mir::ir::GlobalFunctionDefinition::new(definition, false))
    .collect()
}

fn compile_type_information_type() -> mir::types::Record {
    mir::types::Record::new(TYPE_INFORMATION_RECORD_NAME)
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
    use crate::{compile_configuration::COMPILE_CONFIGURATION, error_type};
    use fnv::FnvHashMap;
    use hir::test::{FunctionDefinitionFake, ModuleFake};
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn create_context(module: &Module) -> Context {
        Context::new(module, Some(COMPILE_CONFIGURATION.clone()))
    }

    fn compile_type_information_from_types(
        information: FnvHashMap<mir::types::Type, String>,
    ) -> mir::ir::TypeInformation {
        mir::ir::TypeInformation::new(information, DEFAULT_TYPE_INFORMATION_FUNCTION_NAME.into())
    }

    fn create_default_type_information(context: &Context) -> FnvHashMap<mir::types::Type, String> {
        [
            (
                mir::types::Type::Boolean,
                compile_function_name(context, &types::Boolean::new(Position::fake()).into())
                    .unwrap(),
            ),
            (
                mir::types::Type::ByteString,
                compile_function_name(context, &types::ByteString::new(Position::fake()).into())
                    .unwrap(),
            ),
            (
                mir::types::Type::None,
                compile_function_name(context, &types::None::new(Position::fake()).into()).unwrap(),
            ),
            (
                mir::types::Type::Number,
                compile_function_name(context, &types::Number::new(Position::fake()).into())
                    .unwrap(),
            ),
            (
                error_type::compile_type().into(),
                compile_function_name(context, &types::Error::new(Position::fake()).into())
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
            compile_type_information(&context, &module).unwrap(),
            compile_type_information_from_types(create_default_type_information(&context),)
        );

        for type_ in &[
            types::Boolean::new(Position::fake()).into(),
            types::ByteString::new(Position::fake()).into(),
            types::Error::new(Position::fake()).into(),
            types::None::new(Position::fake()).into(),
            types::Number::new(Position::fake()).into(),
        ] {
            assert!(
                !compile_function_declarations_and_definitions(&context, &module)
                    .unwrap()
                    .1
                    .iter()
                    .find(|definition| definition.definition().name()
                        == debug::compile_function_name(&context, type_).unwrap())
                    .unwrap()
                    .is_public()
            );
        }
    }

    #[test]
    fn compile_without_compile_configuration() {
        let module = Module::empty();
        let context = Context::new(&module, None);

        assert_eq!(
            compile_type_information(&context, &module).unwrap(),
            compile_type_information_from_types(
                [
                    (
                        mir::types::Type::Boolean,
                        compile_function_name(
                            &context,
                            &types::Boolean::new(Position::fake()).into()
                        )
                        .unwrap()
                    ),
                    (
                        mir::types::Type::ByteString,
                        compile_function_name(
                            &context,
                            &types::ByteString::new(Position::fake()).into()
                        )
                        .unwrap()
                    ),
                    (
                        mir::types::Type::None,
                        compile_function_name(&context, &types::None::new(Position::fake()).into())
                            .unwrap()
                    ),
                    (
                        mir::types::Type::Number,
                        compile_function_name(
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
            compile_type_information(&context, &module).unwrap(),
            compile_type_information_from_types(create_default_type_information(&context),)
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
            compile_type_information(&context, &module).unwrap(),
            compile_type_information_from_types(create_default_type_information(&context))
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
            compile_type_information(&context, &module)
                .unwrap()
                .information()
                .len(),
            create_default_type_information(&context).len() + 1
        );
        assert!(
            !compile_function_declarations_and_definitions(&context, &module)
                .unwrap()
                .1
                .iter()
                .find(|definition| definition.definition().name()
                    == debug::compile_function_name(&context, &function_type.clone().into())
                        .unwrap())
                .unwrap()
                .is_public()
        );
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
            compile_type_information(&context, &module)
                .unwrap()
                .information()
                .len(),
            create_default_type_information(&context).len() + 1
        );
        assert!(
            !compile_function_declarations_and_definitions(&context, &module)
                .unwrap()
                .1
                .iter()
                .find(|definition| definition.definition().name()
                    == debug::compile_function_name(&context, &list_type.clone().into()).unwrap())
                .unwrap()
                .is_public()
        );
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
            compile_type_information(&context, &module)
                .unwrap()
                .information()
                .len(),
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
            compile_type_information(&context, &module)
                .unwrap()
                .information()
                .len(),
            create_default_type_information(&context).len() + 1
        );
        assert!(
            !compile_function_declarations_and_definitions(&context, &module)
                .unwrap()
                .1
                .iter()
                .find(|definition| definition.definition().name()
                    == debug::compile_function_name(&context, &map_type.clone().into()).unwrap())
                .unwrap()
                .is_public()
        );
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
            compile_type_information(&context, &module)
                .unwrap()
                .information()
                .len(),
            create_default_type_information(&context).len() + 1
        );
        assert_eq!(
            compile_function_declarations_and_definitions(&context, &module)
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
            compile_type_information(&context, &module)
                .unwrap()
                .information()
                .len(),
            create_default_type_information(&context).len() + 1
        );
        assert!(
            compile_function_declarations_and_definitions(&context, &module)
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
                .is_public()
        );
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
            compile_type_information(&context, &module)
                .unwrap()
                .information()
                .len(),
            create_default_type_information(&context).len() + 2
        );
        assert!(
            !compile_function_declarations_and_definitions(&context, &module)
                .unwrap()
                .1
                .iter()
                .find(|definition| definition.definition().name()
                    == debug::compile_function_name(&context, &field_type.clone().into()).unwrap())
                .unwrap()
                .is_public()
        );
    }
}
