use crate::{concrete_type, context::Context, error_type, type_, type_information, CompileError};
use hir::{
    analysis::{record_field_resolver, type_formatter, type_id_calculator},
    types::Type,
};

const LHS_NAME: &str = "$lhs";
const RHS_NAME: &str = "$rhs";

pub fn compile_call(
    lhs: impl Into<mir::ir::Expression>,
    rhs: impl Into<mir::ir::Expression>,
) -> mir::ir::Expression {
    let lhs = lhs.into();

    mir::ir::Call::new(
        compile_function_type(),
        type_information::compile_function(lhs.clone(), 1),
        vec![lhs.into(), rhs.into()],
    )
    .into()
}

pub(super) fn compile_default_function_name() -> &'static str {
    "hir:equal:default"
}

pub(super) fn compile_function_name(
    context: &Context,
    type_: &Type,
) -> Result<String, CompileError> {
    Ok(format!(
        "hir:equal:{}",
        type_id_calculator::calculate(type_, context.types())?
    ))
}

pub(super) fn compile_function_type() -> mir::types::Function {
    mir::types::Function::new(
        vec![mir::types::Type::Variant, mir::types::Type::Variant],
        // boolean or none
        mir::types::Type::Variant,
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

pub(super) fn compile_function_definition(
    context: &Context,
    type_: &Type,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    let compile_function_definition =
        |body| compile_function_definition_for_concrete_type(context, type_, body);
    let lhs = mir::ir::Variable::new(LHS_NAME);
    let rhs = mir::ir::Variable::new(RHS_NAME);

    Ok(match type_ {
        Type::Boolean(_) => compile_function_definition(
            mir::ir::Variant::new(
                mir::types::Type::Boolean,
                mir::ir::If::new(
                    lhs,
                    rhs,
                    mir::ir::If::new(
                        rhs,
                        mir::ir::Expression::Boolean(false),
                        mir::ir::Expression::Boolean(true),
                    ),
                ),
            )
            .into(),
        )?,
        Type::Error(_) => compile_function_definition(compile_call(
            error_type::compile_source(lhs.into()),
            error_type::compile_source(rhs.into()),
        ))?,
        Type::Function(_) => compile_function_definition(
            mir::ir::Variant::new(mir::types::Type::None, mir::ir::Expression::None).into(),
        )?,
        // TODO
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
                mir::ir::Variable::new(&context.configuration()?.list_type.equal_function_name),
                vec![
                    mir::ir::ByteString::new(type_formatter::format(list_type.element())).into(),
                    compile_unboxed_concrete(context, lhs, type_)?,
                ],
            )
            .into(),
        )?,
        // TODO
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
                mir::ir::Variable::new(&context.configuration()?.map_type.equal_function_name),
                vec![
                    mir::ir::ByteString::new(type_formatter::format(map_type.key())).into(),
                    mir::ir::ByteString::new(type_formatter::format(map_type.value())).into(),
                    compile_unboxed_concrete(context, lhs, type_)?,
                ],
            )
            .into(),
        )?,
        Type::None(_) => compile_function_definition(
            mir::ir::Variant::new(
                mir::types::Type::Boolean,
                mir::ir::Expression::Boolean(true),
            )
            .into(),
        )?,
        Type::Number(_) => compile_function_definition(
            mir::ir::Variant::new(
                mir::types::Type::Boolean,
                mir::ir::ComparisonOperation::new(mir::ir::ComparisonOperator::Equal, lhs, rhs),
            )
            .into(),
        )?,
        // TODO
        Type::Record(record_type) => {
            let mir_type = type_::compile_record(record_type);

            compile_function_definition(
                mir::ir::Variant::new(
                    mir::types::Type::Boolean,
                    record_field_resolver::resolve_record(record_type, context.records())?
                        .iter()
                        .enumerate()
                        .fold(
                            Ok(mir::ir::Expression::Boolean(true)),
                            |result, (index, field)| {
                                let type_ = type_::compile(context, field.type_())?;

                                Ok(mir::ir::If::new(
                                    result?,
                                    mir::ir::Call::new(
                                        type_::compile_concrete(context, field.type_())?,
                                        mir::ir::Variable::new(),
                                        [lhs.clone(), rhs.clone()]
                                            .into_iter()
                                            .map(|value| {
                                                mir::ir::RecordField::new(
                                                    mir_type.clone(),
                                                    index,
                                                    value,
                                                )
                                            })
                                            .collect(),
                                    ),
                                    mir::ir::Expression::Boolean(false),
                                )
                                .into())
                            },
                        )?,
                )
                .into(),
            )?
        }
        Type::String(_) => compile_function_definition(
            mir::ir::Variant::new(
                mir::types::Type::Boolean,
                mir::ir::Call::new(
                    mir::types::Function::new(
                        vec![mir::types::Type::ByteString, mir::types::Type::ByteString],
                        mir::types::Type::Boolean,
                    ),
                    mir::ir::Variable::new(
                        &context.configuration()?.string_type.equal_function_name,
                    ),
                    vec![lhs.into(), rhs.into()],
                ),
            )
            .into(),
        )?,
        Type::Any(_) | Type::Reference(_) | Type::Union(_) => {
            return Err(CompileError::InvalidVariantType(type_.clone()))
        }
    })
}

pub(super) fn compile_default_function_definition() -> mir::ir::FunctionDefinition {
    mir::ir::FunctionDefinition::new(
        compile_default_function_name(),
        vec![
            mir::ir::Argument::new(LHS_NAME, mir::types::Type::Variant),
            mir::ir::Argument::new(RHS_NAME, mir::types::Type::Variant),
        ],
        mir::types::Type::Variant,
        mir::ir::Variant::new(mir::types::Type::None, mir::ir::Expression::None),
    )
}

fn compile_function_definition_for_concrete_type(
    context: &Context,
    type_: &Type,
    body: mir::ir::Expression,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    let mir_type = type_::compile_concrete(context, type_)?;

    Ok(mir::ir::FunctionDefinition::new(
        compile_function_name(context, type_)?,
        vec![
            mir::ir::Argument::new(LHS_NAME, mir::types::Type::Variant),
            mir::ir::Argument::new(RHS_NAME, mir::types::Type::Variant),
        ],
        mir::types::Type::Variant,
        mir::ir::Case::new(
            mir::ir::Variable::new(LHS_NAME),
            vec![mir::ir::Alternative::new(
                vec![mir_type.clone()],
                LHS_NAME,
                mir::ir::Case::new(
                    mir::ir::Variable::new(RHS_NAME),
                    vec![mir::ir::Alternative::new(vec![mir_type], RHS_NAME, body)],
                    Some(mir::ir::DefaultAlternative::new(
                        "",
                        mir::ir::Expression::Boolean(false),
                    )),
                ),
            )],
            Some(mir::ir::DefaultAlternative::new(
                "",
                mir::ir::Expression::Boolean(false),
            )),
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
