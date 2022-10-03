use crate::{concrete_type, context::Context, error_type, type_, CompileError};
use hir::{
    analysis::{record_field_resolver, type_formatter, type_id_calculator},
    types::Type,
};
use itertools::Itertools;

const ARGUMENT_NAME: &str = "$x";

pub fn compile_call(argument: impl Into<mir::ir::Expression>) -> mir::ir::Expression {
    let argument = argument.into();

    mir::ir::Call::new(
        compile_function_type(),
        mir::ir::TypeInformationFunction::new(argument.clone()),
        vec![argument],
    )
    .into()
}

pub(super) fn compile_default_function_name() -> &'static str {
    "hir:debug:default"
}

pub(super) fn compile_function_name(
    context: &Context,
    type_: &Type,
) -> Result<String, CompileError> {
    Ok(format!(
        "hir:debug:{}",
        type_id_calculator::calculate(type_, context.types())?
    ))
}

fn compile_function_type() -> mir::types::Function {
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
) -> Result<Option<mir::ir::FunctionDefinition>, CompileError> {
    let argument = mir::ir::Variable::new(ARGUMENT_NAME);
    let compile_function_definition =
        |body| compile_function_definition_for_concrete_type(context, type_, body);

    Ok(match type_ {
        Type::Boolean(_) => Some(compile_function_definition(
            mir::ir::If::new(
                argument,
                mir::ir::ByteString::new("true"),
                mir::ir::ByteString::new("false"),
            )
            .into(),
        )?),
        Type::Error(_) => Some(compile_function_definition(
            mir::ir::StringConcatenation::new(vec![
                mir::ir::ByteString::new("error(").into(),
                compile_call(error_type::compile_source(argument.into())),
                mir::ir::ByteString::new(")").into(),
            ])
            .into(),
        )?),
        Type::Function(_) => Some(compile_function_definition(
            mir::ir::ByteString::new("<function>").into(),
        )?),
        Type::List(list_type) => Some(compile_function_definition(
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
                    compile_unboxed_concrete(context, argument, type_)?,
                    compile_debug_closure(),
                ],
            )
            .into(),
        )?),
        Type::Map(map_type) => Some(compile_function_definition(
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
                    compile_unboxed_concrete(context, argument, type_)?,
                    compile_debug_closure(),
                ],
            )
            .into(),
        )?),
        Type::None(_) => Some(compile_function_definition(
            mir::ir::ByteString::new("none").into(),
        )?),
        Type::Number(_) => Some(compile_function_definition(
            if let Ok(configuration) = context.configuration() {
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
            },
        )?),
        Type::Record(record_type) => {
            let mir_type = type_::compile_record(record_type);

            Some(compile_function_definition(
                mir::ir::StringConcatenation::new(
                    [mir::ir::ByteString::new(format!("{}{{", record_type.name())).into()]
                        .into_iter()
                        .chain(
                            record_field_resolver::resolve_record(record_type, context.records())?
                                .iter()
                                .enumerate()
                                .map(|(index, field)| {
                                    let type_ = type_::compile(context, field.type_())?;
                                    let value = mir::ir::RecordField::new(
                                        mir_type.clone(),
                                        index,
                                        argument.clone(),
                                    );

                                    Ok(vec![
                                        mir::ir::ByteString::new(format!("{}: ", field.name()))
                                            .into(),
                                        compile_call(if type_ == mir::types::Type::Variant {
                                            mir::ir::Expression::from(value)
                                        } else {
                                            mir::ir::Variant::new(
                                                type_::compile_concrete(context, field.type_())?,
                                                concrete_type::compile(
                                                    context,
                                                    value.into(),
                                                    field.type_(),
                                                )?,
                                            )
                                            .into()
                                        }),
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
            )?)
        }
        Type::String(_) => Some(compile_function_definition(
            mir::ir::StringConcatenation::new(vec![
                mir::ir::ByteString::new("\"").into(),
                argument.into(),
                mir::ir::ByteString::new("\"").into(),
            ])
            .into(),
        )?),
        Type::Any(_) | Type::Reference(_) | Type::Union(_) => None,
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

fn compile_unboxed_concrete(
    context: &Context,
    expression: impl Into<mir::ir::Expression>,
    type_: &Type,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(mir::ir::RecordField::new(
        type_::compile_concrete(context, type_)?
            .into_record()
            .unwrap(),
        0,
        expression.into(),
    )
    .into())
}

fn compile_debug_closure() -> mir::ir::Expression {
    const CLOSURE_NAME: &str = "hir:debug:closure";

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
    use position::{test::PositionFake, Position};
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
        let definition = compile_function_definition(&context, &type_)
            .unwrap()
            .unwrap();

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
        let definition = compile_function_definition(&context, &type_)
            .unwrap()
            .unwrap();

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
        let definition = compile_function_definition(&context, &type_)
            .unwrap()
            .unwrap();

        assert_eq!(
            definition.name(),
            &compile_function_name(&context, &type_).unwrap()
        );
        assert_eq!(definition.type_(), &compile_function_type());
    }
}
