use super::utility;
use crate::{CompileError, context::Context, error_type, type_, type_information};
use hir::{
    analysis::{record_field_resolver, type_formatter, type_id_calculator},
    types::Type,
};
use itertools::Itertools;

const FUNCTION_PREFIX: &str = "hir:reflect:debug:";
const ARGUMENT_NAME: &str = "$x";

pub fn compile_call(argument: impl Into<mir::ir::Expression>) -> mir::ir::Expression {
    let argument = argument.into();

    mir::ir::Call::new(
        compile_function_type(),
        type_information::compile_function(argument.clone(), 0),
        vec![argument],
    )
    .into()
}

pub(super) fn compile_default_function_name() -> String {
    FUNCTION_PREFIX.to_owned() + "default"
}

pub(super) fn compile_function_name(
    context: &Context,
    type_: &Type,
) -> Result<String, CompileError> {
    Ok(FUNCTION_PREFIX.to_owned() + &type_id_calculator::calculate(type_, context.types())?)
}

pub(super) fn compile_function_type() -> mir::types::Function {
    mir::types::Function::new(
        vec![mir::types::Type::Variant],
        mir::types::Type::ByteString,
    )
}

pub(super) fn compile_function_declaration(
    context: &Context,
    type_: &Type,
) -> Result<mir::ir::FunctionDeclaration, CompileError> {
    Ok(mir::ir::FunctionDeclaration::new(
        compile_function_name(context, type_)?,
        compile_function_type(),
    ))
}

#[allow(unstable_name_collisions)]
pub(super) fn compile_function_definition(
    context: &Context,
    type_: &Type,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    let argument = mir::ir::Variable::new(ARGUMENT_NAME);
    let compile_function_definition =
        |body| compile_function_definition_for_concrete_type(context, type_, body);

    Ok(match type_ {
        Type::Boolean(_) => compile_function_definition(
            mir::ir::If::new(
                argument,
                mir::ir::ByteString::new("true"),
                mir::ir::ByteString::new("false"),
            )
            .into(),
        )?,
        Type::Error(_) => compile_function_definition(
            mir::ir::StringConcatenation::new(vec![
                mir::ir::ByteString::new("error(").into(),
                compile_call(error_type::compile_source(argument.into())),
                mir::ir::ByteString::new(")").into(),
            ])
            .into(),
        )?,
        Type::Function(_) => {
            compile_function_definition(mir::ir::ByteString::new("<function>").into())?
        }
        Type::List(list_type) => compile_function_definition(
            mir::ir::Call::new(
                mir::types::Function::new(
                    vec![
                        mir::types::Type::ByteString,
                        type_::compile_list(context)?.into(),
                        compile_function_type().into(),
                    ],
                    mir::types::Type::ByteString,
                ),
                mir::ir::Variable::new(&context.configuration()?.list_type.debug_function_name),
                vec![
                    mir::ir::ByteString::new(type_formatter::format(list_type.element())).into(),
                    utility::compile_unboxed_concrete(context, argument, type_)?,
                    compile_element_function(),
                ],
            )
            .into(),
        )?,
        Type::Map(map_type) => compile_function_definition(
            mir::ir::Call::new(
                mir::types::Function::new(
                    vec![
                        mir::types::Type::ByteString,
                        mir::types::Type::ByteString,
                        type_::compile_map(context)?.into(),
                        compile_function_type().into(),
                    ],
                    mir::types::Type::ByteString,
                ),
                mir::ir::Variable::new(&context.configuration()?.map_type.debug_function_name),
                vec![
                    mir::ir::ByteString::new(type_formatter::format(map_type.key())).into(),
                    mir::ir::ByteString::new(type_formatter::format(map_type.value())).into(),
                    utility::compile_unboxed_concrete(context, argument, type_)?,
                    compile_element_function(),
                ],
            )
            .into(),
        )?,
        Type::None(_) => compile_function_definition(mir::ir::ByteString::new("none").into())?,
        Type::Number(_) => {
            compile_function_definition(if let Ok(configuration) = context.configuration() {
                mir::ir::Call::new(
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Type::ByteString,
                    ),
                    mir::ir::Variable::new(&configuration.number_type.debug_function_name),
                    vec![argument.into()],
                )
                .into()
            } else {
                mir::ir::ByteString::new("<number>").into()
            })?
        }
        Type::Record(record_type) => {
            let mir_type = type_::compile_record(record_type);

            compile_function_definition(
                mir::ir::StringConcatenation::new(
                    [
                        mir::ir::ByteString::new(format!("{}{{", record_type.original_name()))
                            .into(),
                    ]
                    .into_iter()
                    .chain(
                        record_field_resolver::resolve_record(record_type, context.records())?
                            .iter()
                            .enumerate()
                            .map(|(index, field)| {
                                let value = mir::ir::RecordField::new(
                                    mir_type.clone(),
                                    index,
                                    argument.clone(),
                                );

                                Ok(vec![
                                    mir::ir::ByteString::new(format!("{}: ", field.name())).into(),
                                    compile_call(utility::compile_any(
                                        context,
                                        value,
                                        field.type_(),
                                    )?),
                                ])
                            })
                            .collect::<Result<Vec<_>, CompileError>>()?
                            .into_iter()
                            .intersperse(vec![mir::ir::ByteString::new(", ").into()])
                            .flatten(),
                    )
                    .chain([mir::ir::ByteString::new("}").into()])
                    .collect(),
                )
                .into(),
            )?
        }
        Type::String(_) => compile_function_definition(
            mir::ir::StringConcatenation::new(vec![
                mir::ir::ByteString::new("\"").into(),
                argument.into(),
                mir::ir::ByteString::new("\"").into(),
            ])
            .into(),
        )?,
        Type::Any(_) | Type::Reference(_) | Type::Union(_) => {
            return Err(CompileError::InvalidVariantType(type_.clone()));
        }
    })
}

pub(super) fn compile_default_function_definition() -> mir::ir::FunctionDefinition {
    mir::ir::FunctionDefinition::new(
        compile_default_function_name(),
        vec![mir::ir::Argument::new(
            ARGUMENT_NAME,
            mir::types::Type::Variant,
        )],
        mir::types::Type::ByteString,
        mir::ir::ByteString::new("<unknown>"),
    )
}

fn compile_function_definition_for_concrete_type(
    context: &Context,
    type_: &Type,
    body: mir::ir::Expression,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    Ok(mir::ir::FunctionDefinition::new(
        compile_function_name(context, type_)?,
        vec![mir::ir::Argument::new(
            ARGUMENT_NAME,
            mir::types::Type::Variant,
        )],
        mir::types::Type::ByteString,
        mir::ir::Case::new(
            mir::ir::Variable::new(ARGUMENT_NAME),
            vec![mir::ir::Alternative::new(
                vec![type_::compile_concrete(context, type_)?],
                ARGUMENT_NAME,
                body,
            )],
            None,
        ),
    ))
}

fn compile_element_function() -> mir::ir::Expression {
    const CLOSURE_NAME: &str = "hir:debug:element";

    mir::ir::LetRecursive::new(
        mir::ir::FunctionDefinition::new(
            CLOSURE_NAME,
            vec![mir::ir::Argument::new(
                ARGUMENT_NAME,
                mir::types::Type::Variant,
            )],
            mir::types::Type::ByteString,
            compile_call(mir::ir::Variable::new(ARGUMENT_NAME)),
        ),
        mir::ir::Variable::new(CLOSURE_NAME),
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::{ir::*, test::ModuleFake, types};
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    #[test]
    fn compile_default_function_definition_() {
        let definition = compile_default_function_definition();

        assert_eq!(definition.name(), compile_default_function_name());
        assert_eq!(definition.type_(), &compile_function_type());
    }

    #[test]
    fn compile_function_definition_for_none() {
        let context = Context::new(&Module::empty(), None);
        let type_ = types::None::new(Position::fake()).into();
        let definition = compile_function_definition(&context, &type_).unwrap();

        assert_eq!(
            definition.name(),
            &compile_function_name(&context, &type_).unwrap()
        );
        assert_eq!(definition.type_(), &compile_function_type());
    }

    #[test]
    fn compile_function_definition_for_number() {
        let context = Context::new(&Module::empty(), Some(COMPILE_CONFIGURATION.clone()));
        let type_ = types::Number::new(Position::fake()).into();
        let definition = compile_function_definition(&context, &type_).unwrap();

        assert_eq!(
            definition.name(),
            &compile_function_name(&context, &type_).unwrap()
        );
        assert_eq!(definition.type_(), &compile_function_type());
    }

    #[test]
    fn compile_function_definition_for_number_without_configuration() {
        let context = Context::new(&Module::empty(), None);
        let type_ = types::None::new(Position::fake()).into();
        let definition = compile_function_definition(&context, &type_).unwrap();

        assert_eq!(
            definition.name(),
            &compile_function_name(&context, &type_).unwrap()
        );
        assert_eq!(definition.type_(), &compile_function_type());
    }

    #[test]
    fn compile_function_definition_for_any() {
        let context = Context::new(&Module::empty(), None);
        let type_ = types::Any::new(Position::fake()).into();

        assert_eq!(
            compile_function_definition(&context, &type_),
            Err(CompileError::InvalidVariantType(type_))
        );
    }
}
