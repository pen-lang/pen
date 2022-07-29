use super::hash_calculation;
use crate::{context::CompileContext, transformation::record_type_information, CompileError};
use hir::{
    analysis::type_comparability_checker,
    ir::*,
    types::{self, Type},
};
use position::Position;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

const RECORD_NAME: &str = "$r";

pub fn transform(context: &CompileContext, module: &Module) -> Result<Module, CompileError> {
    // We cannot define hash functions for record types if hash configuration is not
    // available.
    if context.configuration().is_err() {
        return Ok(module.clone());
    }

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

        if type_definition.is_external() {
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
            .function_declarations()
            .iter()
            .cloned()
            .chain(function_declarations)
            .collect(),
        module
            .function_definitions()
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
) -> Result<FunctionDefinition, CompileError> {
    let position = type_definition.position();
    let record_type = types::Record::new(type_definition.name(), position.clone());
    let function_name = record_type_information::compile_hash_function_name(&record_type);
    let hash_type = compile_hash_type(position);
    let configuration = &context.configuration()?.map_type.hash;

    Ok(FunctionDefinition::new(
        &function_name,
        &function_name,
        Lambda::new(
            vec![Argument::new(RECORD_NAME, record_type.clone())],
            hash_type.clone(),
            type_definition.fields().iter().rev().fold(
                Ok(Expression::from(compile_identity_hash(type_definition))),
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
                            hash_calculation::transform(
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

fn compile_hash_function_declaration(type_definition: &TypeDefinition) -> FunctionDeclaration {
    let position = type_definition.position();
    let record_type = types::Record::new(type_definition.name(), position.clone());

    FunctionDeclaration::new(
        record_type_information::compile_hash_function_name(&record_type),
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

// TODO Collision of these hashes might lead to infinite loop in built-in map
// type insertion because they are treated as identities there.
fn compile_identity_hash(type_definition: &TypeDefinition) -> Number {
    let mut hasher = DefaultHasher::new();

    type_definition.name().hash(&mut hasher);

    Number::new(
        f64::from_bits(hasher.finish()),
        type_definition.position().clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::test::ModuleFake;
    use once_cell::sync::Lazy;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    static HASH_TYPE: Lazy<Type> = Lazy::new(|| compile_hash_type(&Position::fake()));
    static COMBINE_HASH_FUNCTION_TYPE: Lazy<Type> = Lazy::new(|| {
        types::Function::new(
            vec![HASH_TYPE.clone(), HASH_TYPE.clone()],
            HASH_TYPE.clone(),
            Position::fake(),
        )
        .into()
    });

    fn transform_module(module: &Module) -> Result<Module, CompileError> {
        transform(
            &CompileContext::new(module, COMPILE_CONFIGURATION.clone().into()),
            module,
        )
    }

    #[test]
    fn compile_hash_function() {
        let type_definition = TypeDefinition::new(
            "foo",
            "foo",
            vec![
                types::RecordField::new("x", types::None::new(Position::fake())),
                types::RecordField::new("y", types::None::new(Position::fake())),
            ],
            false,
            false,
            false,
            Position::fake(),
        );
        let record_type = types::Record::new(type_definition.name(), Position::fake());

        assert_eq!(
            transform_module(&Module::empty().set_type_definitions(vec![type_definition.clone()])),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition.clone()])
                .set_function_definitions(vec![FunctionDefinition::new(
                    "foo.$hash",
                    "foo.$hash",
                    Lambda::new(
                        vec![Argument::new(RECORD_NAME, record_type),],
                        HASH_TYPE.clone(),
                        Call::new(
                            Some(COMBINE_HASH_FUNCTION_TYPE.clone()),
                            Variable::new(
                                &COMPILE_CONFIGURATION.map_type.hash.combine_function_name,
                                Position::fake()
                            ),
                            vec![
                                Call::new(
                                    Some(COMBINE_HASH_FUNCTION_TYPE.clone()),
                                    Variable::new(
                                        &COMPILE_CONFIGURATION.map_type.hash.combine_function_name,
                                        Position::fake()
                                    ),
                                    vec![
                                        compile_identity_hash(&type_definition).into(),
                                        Number::new(0.0, Position::fake()).into(),
                                    ],
                                    Position::fake()
                                )
                                .into(),
                                Number::new(0.0, Position::fake()).into(),
                            ],
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    None,
                    false,
                    Position::fake()
                )]))
        );
    }

    #[test]
    fn compile_hash_function_declaration_for_external_type_definition() {
        let type_definition = TypeDefinition::new(
            "foo",
            "foo",
            vec![
                types::RecordField::new("x", types::None::new(Position::fake())),
                types::RecordField::new("y", types::None::new(Position::fake())),
            ],
            false,
            false,
            true,
            Position::fake(),
        );

        assert_eq!(
            transform_module(&Module::empty().set_type_definitions(vec![type_definition.clone()])),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition.clone()])
                .set_function_declarations(vec![FunctionDeclaration::new(
                    "foo.$hash",
                    types::Function::new(
                        vec![types::Record::new(type_definition.name(), Position::fake()).into()],
                        HASH_TYPE.clone(),
                        Position::fake()
                    ),
                    Position::fake()
                )]))
        );
    }
}
