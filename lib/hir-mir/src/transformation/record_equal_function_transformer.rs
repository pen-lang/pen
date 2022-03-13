use crate::{
    context::CompileContext, transformation::record_type_information_compiler, CompileError,
};
use hir::{analysis::type_comparability_checker, ir::*, types};

const LHS_NAME: &str = "$lhs";
const RHS_NAME: &str = "$rhs";

pub fn transform(context: &CompileContext, module: &Module) -> Result<Module, CompileError> {
    let mut equal_function_definitions = vec![];
    let mut equal_function_declarations = vec![];

    for type_definition in module.type_definitions() {
        if !type_comparability_checker::check(
            &types::Record::new(type_definition.name(), type_definition.position().clone()).into(),
            context.types(),
            context.records(),
        )? {
            continue;
        }

        if type_definition.is_external() {
            equal_function_declarations.push(compile_equal_function_declaration(type_definition));
        } else {
            equal_function_definitions.push(compile_equal_function_definition(type_definition));
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
            .chain(equal_function_declarations)
            .collect(),
        module
            .definitions()
            .iter()
            .cloned()
            .chain(equal_function_definitions)
            .collect(),
        module.position().clone(),
    ))
}

fn compile_equal_function_definition(type_definition: &TypeDefinition) -> Definition {
    let position = type_definition.position();
    let record_type = types::Record::new(type_definition.name(), position.clone());

    let function_name = record_type_information_compiler::compile_equal_function_name(&record_type);

    Definition::new(
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
        false,
        position.clone(),
    )
}

fn compile_equal_function_declaration(type_definition: &TypeDefinition) -> Declaration {
    let position = type_definition.position();
    let record_type = types::Record::new(type_definition.name(), position.clone());

    Declaration::new(
        record_type_information_compiler::compile_equal_function_name(&record_type),
        types::Function::new(
            vec![record_type.clone().into(), record_type.clone().into()],
            types::Boolean::new(position.clone()),
            position.clone(),
        ),
        position.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::test::ModuleFake;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn transform_module(module: &Module) -> Result<Module, CompileError> {
        transform(
            &CompileContext::new(module, COMPILE_CONFIGURATION.clone().into()),
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
        let record_type = types::Record::new(type_definition.name(), Position::fake());

        assert_eq!(
            transform_module(&Module::empty().set_type_definitions(vec![type_definition.clone()])),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_definitions(vec![Definition::new(
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
                    false,
                    Position::fake()
                )]))
        );
    }

    #[test]
    fn compile_equal_function_declaration_for_external_type_definition() {
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
        let record_type = types::Record::new(type_definition.name(), Position::fake());

        assert_eq!(
            transform_module(&Module::empty().set_type_definitions(vec![type_definition.clone()])),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_declarations(vec![Declaration::new(
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
