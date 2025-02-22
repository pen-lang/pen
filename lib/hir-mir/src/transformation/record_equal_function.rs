use crate::{CompileError, context::Context, transformation::record_type_information};
use hir::{analysis::type_comparability_checker, ir::*, types};

const LHS_NAME: &str = "$lhs";
const RHS_NAME: &str = "$rhs";

pub fn transform(context: &Context, module: &Module) -> Result<Module, CompileError> {
    let (external_type_definitions, internal_type_definitions) = module
        .type_definitions()
        .iter()
        .map(|definition| {
            Ok(
                if type_comparability_checker::check(
                    &types::Record::new(
                        definition.name(),
                        definition.original_name(),
                        definition.position().clone(),
                    )
                    .into(),
                    context.types(),
                    context.records(),
                )? {
                    Some(definition)
                } else {
                    None
                },
            )
        })
        .collect::<Result<Vec<_>, CompileError>>()?
        .into_iter()
        .flatten()
        .partition::<Vec<_>, _>(|definition| definition.is_external());

    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module
            .function_declarations()
            .iter()
            .cloned()
            .chain(
                external_type_definitions
                    .iter()
                    .copied()
                    .map(compile_function_declaration),
            )
            .collect(),
        module
            .function_definitions()
            .iter()
            .cloned()
            .chain(
                internal_type_definitions
                    .iter()
                    .copied()
                    .map(compile_function_definition),
            )
            .collect(),
        module.position().clone(),
    ))
}

fn compile_function_declaration(type_definition: &TypeDefinition) -> FunctionDeclaration {
    let position = type_definition.position();
    let record_type = types::Record::new(
        type_definition.name(),
        type_definition.original_name(),
        position.clone(),
    );

    FunctionDeclaration::new(
        record_type_information::compile_equal_function_name(&record_type),
        types::Function::new(
            vec![record_type.clone().into(), record_type.clone().into()],
            types::Boolean::new(position.clone()),
            position.clone(),
        ),
        position.clone(),
    )
}

fn compile_function_definition(type_definition: &TypeDefinition) -> FunctionDefinition {
    let position = type_definition.position();
    let record_type = types::Record::new(
        type_definition.name(),
        type_definition.original_name(),
        position.clone(),
    );

    let function_name = record_type_information::compile_equal_function_name(&record_type);

    FunctionDefinition::new(
        &function_name,
        &function_name,
        Lambda::new(
            vec![
                Argument::new(LHS_NAME, record_type.clone()),
                Argument::new(RHS_NAME, record_type.clone()),
            ],
            types::Boolean::new(position.clone()),
            type_definition.fields().iter().rev().fold(
                Boolean::new(true, position.clone()).into(),
                |expression: Expression, field| {
                    If::new(
                        EqualityOperation::new(
                            Some(field.type_().clone()),
                            EqualityOperator::Equal,
                            RecordDeconstruction::new(
                                Some(record_type.clone().into()),
                                Variable::new(LHS_NAME, position.clone()),
                                field.name(),
                                position.clone(),
                            ),
                            RecordDeconstruction::new(
                                Some(record_type.clone().into()),
                                Variable::new(RHS_NAME, position.clone()),
                                field.name(),
                                position.clone(),
                            ),
                            position.clone(),
                        ),
                        expression,
                        Boolean::new(false, position.clone()),
                        position.clone(),
                    )
                    .into()
                },
            ),
            position.clone(),
        ),
        None,
        true,
        position.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::test::{ModuleFake, RecordFake};
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    fn transform_module(module: &Module) -> Result<Module, CompileError> {
        transform(
            &Context::new(module, COMPILE_CONFIGURATION.clone().into()),
            module,
        )
    }

    #[test]
    fn compile_equal_function() {
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
        let record_type = types::Record::fake(type_definition.name());

        assert_eq!(
            transform_module(&Module::empty().set_type_definitions(vec![type_definition.clone()])),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_function_definitions(vec![FunctionDefinition::new(
                    "foo.$equal",
                    "foo.$equal",
                    Lambda::new(
                        vec![
                            Argument::new(LHS_NAME, record_type.clone()),
                            Argument::new(RHS_NAME, record_type.clone()),
                        ],
                        types::Boolean::new(Position::fake()),
                        If::new(
                            EqualityOperation::new(
                                Some(types::None::new(Position::fake()).into()),
                                EqualityOperator::Equal,
                                RecordDeconstruction::new(
                                    Some(record_type.clone().into()),
                                    Variable::new(LHS_NAME, Position::fake()),
                                    "x",
                                    Position::fake(),
                                ),
                                RecordDeconstruction::new(
                                    Some(record_type.clone().into()),
                                    Variable::new(RHS_NAME, Position::fake()),
                                    "x",
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            If::new(
                                EqualityOperation::new(
                                    Some(types::None::new(Position::fake()).into()),
                                    EqualityOperator::Equal,
                                    RecordDeconstruction::new(
                                        Some(record_type.clone().into()),
                                        Variable::new(LHS_NAME, Position::fake()),
                                        "y",
                                        Position::fake(),
                                    ),
                                    RecordDeconstruction::new(
                                        Some(record_type.into()),
                                        Variable::new(RHS_NAME, Position::fake()),
                                        "y",
                                        Position::fake(),
                                    ),
                                    Position::fake(),
                                ),
                                Boolean::new(true, Position::fake()),
                                Boolean::new(false, Position::fake()),
                                Position::fake(),
                            ),
                            Boolean::new(false, Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    None,
                    true,
                    Position::fake()
                )]))
        );
    }

    #[test]
    fn compile_equal_function_declaration_for_external_type_definition() {
        let record_type = types::Record::fake("foo");
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
                .set_type_definitions(vec![type_definition])
                .set_function_declarations(vec![FunctionDeclaration::new(
                    "foo.$equal",
                    types::Function::new(
                        vec![record_type.clone().into(), record_type.into()],
                        types::Boolean::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
                )]))
        );
    }
}
