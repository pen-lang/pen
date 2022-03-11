use super::hash_calculation_transformer;
use crate::{
    context::CompileContext, transformation::record_type_information_compiler, CompileError,
};
use hir::{
    analysis::type_comparability_checker,
    ir::*,
    types::{self, Type},
};
use position::Position;

const RECORD_NAME: &str = "$x";

pub fn transform(context: &CompileContext, module: &Module) -> Result<Module, CompileError> {
    let mut function_definitions = vec![];
    let mut function_declarations = vec![];

    for type_definition in module.type_definitions() {
        if !type_comparability_checker::check(
            &types::Record::new(type_definition.name(), type_definition.position().clone()).into(),
            context.types(),
            context.records(),
        )? {
            continue;
        }

        if type_definition.is_external()
            && type_comparability_checker::check(
                &types::Record::new(type_definition.name(), type_definition.position().clone())
                    .into(),
                context.types(),
                context.records(),
            )?
        {
            function_declarations.push(compile_hash_function_declaration(type_definition));
        } else {
            function_definitions.push(compile_hash_function_definition(context, type_definition)?);
        }
    }

    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module
            .declarations()
            .iter()
            .cloned()
            .chain(function_declarations)
            .collect(),
        module
            .definitions()
            .iter()
            .cloned()
            .chain(function_definitions)
            .collect(),
        module.position().clone(),
    ))
}

fn compile_hash_function_definition(
    context: &CompileContext,
    type_definition: &TypeDefinition,
) -> Result<Definition, CompileError> {
    let position = type_definition.position();
    let record_type = types::Record::new(type_definition.name(), position.clone());
    let function_name = record_type_information_compiler::compile_hash_function_name(&record_type);
    let hash_type = compile_hash_type(position);
    let configuration = &context.configuration()?.map_type.hash;

    Ok(Definition::new(
        &function_name,
        &function_name,
        Lambda::new(
            vec![Argument::new(RECORD_NAME, record_type.clone())],
            hash_type.clone(),
            type_definition.fields().iter().rev().fold(
                Ok(Expression::from(Number::new(0.0, position.clone()))),
                |expression, field| -> Result<_, CompileError> {
                    Ok(Call::new(
                        Some(
                            types::Function::new(
                                vec![hash_type.clone(), hash_type.clone()],
                                hash_type.clone(),
                                position.clone(),
                            )
                            .into(),
                        ),
                        Variable::new(&configuration.combine_function_name, position.clone()),
                        vec![
                            expression?,
                            hash_calculation_transformer::transform(
                                context,
                                &RecordDeconstruction::new(
                                    Some(record_type.clone().into()),
                                    Variable::new(RECORD_NAME, position.clone()),
                                    field.name(),
                                    position.clone(),
                                )
                                .into(),
                                field.type_(),
                                position,
                            )?,
                        ],
                        position.clone(),
                    )
                    .into())
                },
            )?,
            position.clone(),
        ),
        None,
        false,
        position.clone(),
    ))
}

fn compile_hash_function_declaration(type_definition: &TypeDefinition) -> Declaration {
    let position = type_definition.position();
    let record_type = types::Record::new(type_definition.name(), position.clone());

    Declaration::new(
        record_type_information_compiler::compile_hash_function_name(&record_type),
        types::Function::new(
            vec![record_type.clone().into()],
            compile_hash_type(position),
            position.clone(),
        ),
        position.clone(),
    )
}

fn compile_hash_type(position: &Position) -> Type {
    types::Number::new(position.clone()).into()
}
