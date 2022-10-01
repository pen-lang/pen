use crate::{context::Context, type_, CompileError};
use hir::{analysis::type_id_calculator, types::Type};

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

pub(super) fn compile_function_definition(
    context: &Context,
    type_: &Type,
) -> Result<Option<mir::ir::FunctionDefinition>, CompileError> {
    let argument = mir::ir::Variable::new(ARGUMENT_NAME);
    let compile_function_definition =
        |body| compile_function_definition_for_concrete_type(context, type_, body);

    // TODO Implement proper type-specific debug format.
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

#[cfg(test)]
mod tests {
    use super::*;
    use hir::{ir::*, test::ModuleFake, types};
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

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
    fn compile_default_function_definition_() {
        let definition = compile_default_function_definition();

        assert_eq!(definition.name(), compile_default_function_name());
        assert_eq!(definition.type_(), &compile_function_type());
    }
}
