use crate::{context::CompileContext, type_, CompileError};
use hir::types::Type;

const ARGUMENT_NAME: &str = "$x";

pub fn compile_call(argument: impl Into<mir::ir::Expression>) -> mir::ir::Expression {
    let argument = argument.into();

    mir::ir::Call::new(
        compile_function_type(),
        mir::ir::TypeInformationFunction::new(super::DEBUG_FUNCTION_INDEX, argument.clone()),
        vec![argument],
    )
    .into()
}

pub(super) fn compile_function_name(
    context: &CompileContext,
    type_: &Type,
) -> Result<String, CompileError> {
    Ok(format!(
        "hir:debug:{}",
        mir::analysis::type_id::calculate(&type_::compile_concrete(context, type_)?)
    ))
}

pub(super) fn compile_function_type() -> mir::types::Function {
    mir::types::Function::new(
        vec![mir::types::Type::Variant],
        mir::types::Type::ByteString,
    )
}

pub(super) fn compile_function_definition(
    context: &CompileContext,
    type_: &Type,
) -> Result<Option<mir::ir::FunctionDefinition>, CompileError> {
    let argument = mir::ir::Variable::new(ARGUMENT_NAME);
    let compile_function_definition =
        |body| compile_function_definition_for_non_variant(context, type_, body);

    // TODO Implement proper type-specfic debug format.
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
                compile_call(mir::ir::RecordField::new(
                    type_::compile_error(context)?,
                    0,
                    argument,
                )),
                mir::ir::ByteString::new(")").into(),
            ])
            .into(),
        )?),
        Type::Function(_) => Some(compile_function_definition(
            mir::ir::ByteString::new("<function>").into(),
        )?),
        Type::List(_) => Some(compile_function_definition(
            mir::ir::ByteString::new("<list>").into(),
        )?),
        Type::Map(_) => Some(compile_function_definition(
            mir::ir::ByteString::new("<map>").into(),
        )?),
        Type::None(_) => Some(compile_function_definition(
            mir::ir::ByteString::new("none").into(),
        )?),
        Type::Number(_) => Some(compile_function_definition(
            mir::ir::ByteString::new("<number>").into(),
        )?),
        Type::Record(_) => Some(compile_function_definition(
            mir::ir::ByteString::new("<record>").into(),
        )?),
        Type::Reference(_) => unreachable!(),
        Type::String(_) => Some(compile_function_definition(
            mir::ir::StringConcatenation::new(vec![
                mir::ir::ByteString::new("\"").into(),
                argument.into(),
                mir::ir::ByteString::new("\"").into(),
            ])
            .into(),
        )?),
        Type::Any(_) | Type::Union(_) => None,
    })
}

fn compile_function_definition_for_non_variant(
    context: &CompileContext,
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
