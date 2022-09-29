use crate::{context::CompileContext, type_, CompileError};
use hir::{analysis::type_id_calculator, types::Type};

const ARGUMENT_NAME: &str = "$x";

pub fn compile_function_type() -> mir::types::Function {
    mir::types::Function::new(
        vec![mir::types::Type::Variant],
        mir::types::Type::ByteString,
    )
}

pub fn compile_call(argument: mir::ir::Expression) -> mir::ir::Expression {
    mir::ir::Call::new(
        compile_function_type(),
        mir::ir::TypeInformationFunction::new(super::DEBUG_FUNCTION_INDEX, argument.clone()),
        vec![argument],
    )
    .into()
}

pub fn compile_function_name(
    context: &CompileContext,
    type_: &Type,
) -> Result<String, CompileError> {
    Ok(format!(
        "hir:debug:{}",
        type_id_calculator::calculate(type_, context.types())?
    ))
}

pub fn compile_function_definition(
    context: &CompileContext,
    type_: &Type,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    let argument = mir::ir::Variable::new(ARGUMENT_NAME);
    let mir_type = type_::compile(context, type_)?;
    let compile_downcast = |body| compile_downcast(argument.clone().into(), mir_type.clone(), body);

    // TODO Compile type information.
    Ok(mir::ir::FunctionDefinition::new(
        compile_function_name(context, type_)?,
        vec![mir::ir::Argument::new(
            ARGUMENT_NAME,
            mir::types::Type::Variant,
        )],
        mir::types::Type::ByteString,
        match type_ {
            Type::Any(_) => compile_call(argument.into()),
            Type::Boolean(_) => compile_downcast(
                mir::ir::If::new(
                    argument.clone(),
                    mir::ir::ByteString::new("true"),
                    mir::ir::ByteString::new("false"),
                )
                .into(),
            ),
            Type::Error(_) => mir::ir::StringConcatenation::new(vec![
                mir::ir::ByteString::new("error(").into(),
                compile_downcast(compile_call(
                    mir::ir::RecordField::new(type_::compile_error(context)?, 0, argument.clone())
                        .into(),
                )),
                mir::ir::ByteString::new(")").into(),
            ])
            .into(),
            Type::Function(_) => mir::ir::ByteString::new("<function>").into(),
            Type::List(_) => mir::ir::ByteString::new("<list>").into(),
            Type::Map(_) => mir::ir::ByteString::new("<map>").into(),
            Type::None(_) => mir::ir::ByteString::new("none").into(),
            Type::Number(_) => mir::ir::ByteString::new("<number>").into(),
            Type::Record(_) => mir::ir::ByteString::new("<record>").into(),
            Type::Reference(_) => mir::ir::ByteString::new("<reference>").into(),
            Type::String(_) => mir::ir::StringConcatenation::new(vec![
                mir::ir::ByteString::new("\"").into(),
                compile_downcast(argument.clone().into()),
                mir::ir::ByteString::new("\"").into(),
            ])
            .into(),
            Type::Union(_) => return Err(CompileError::UnsupportedTypeInformation(type_.clone())),
        },
    ))
}

fn compile_downcast(
    argument: mir::ir::Expression,
    type_: mir::types::Type,
    body: mir::ir::Expression,
) -> mir::ir::Expression {
    mir::ir::Case::new(
        argument,
        vec![mir::ir::Alternative::new(vec![type_], ARGUMENT_NAME, body)],
        None,
    )
    .into()
}
