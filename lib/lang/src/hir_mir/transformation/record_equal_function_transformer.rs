use crate::hir::*;
use crate::hir_mir::transformation::record_type_information_compiler;
use crate::hir_mir::type_context::TypeContext;
use crate::hir_mir::CompileError;
use crate::types;
use crate::types::analysis::type_comparability_checker;

const LHS_NAME: &str = "$lhs";
const RHS_NAME: &str = "$rhs";

pub fn transform(module: &Module, type_context: &TypeContext) -> Result<Module, CompileError> {
    let mut equal_function_definitions = vec![];

    for type_definition in module.type_definitions() {
        if type_comparability_checker::check(
            &types::Record::new(type_definition.name(), type_definition.position().clone()).into(),
            type_context.types(),
            type_context.records(),
        )? {
            equal_function_definitions.push(compile_record_equal_function(type_definition));
        }
    }

    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .cloned()
            .chain(equal_function_definitions)
            .collect(),
    ))
}

fn compile_record_equal_function(type_definition: &TypeDefinition) -> Definition {
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
            type_definition.elements().iter().rev().fold(
                Boolean::new(true, position.clone()).into(),
                |expression: Expression, element| {
                    If::new(
                        EqualityOperation::new(
                            Some(element.type_().clone()),
                            EqualityOperator::Equal,
                            RecordDeconstruction::new(
                                Some(record_type.clone().into()),
                                Variable::new(LHS_NAME, position.clone()),
                                element.name(),
                                position.clone(),
                            ),
                            RecordDeconstruction::new(
                                Some(record_type.clone().into()),
                                Variable::new(RHS_NAME, position.clone()),
                                element.name(),
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
        false,
        false,
        position.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hir_mir::{
            list_type_configuration::LIST_TYPE_CONFIGURATION,
            string_type_configuration::STRING_TYPE_CONFIGURATION,
        },
        position::Position,
    };

    fn transform_module(module: &Module) -> Result<Module, CompileError> {
        transform(
            module,
            &TypeContext::new(module, &LIST_TYPE_CONFIGURATION, &STRING_TYPE_CONFIGURATION),
        )
    }

    #[test]
    fn compile_equal_function() {
        let type_definition = TypeDefinition::new(
            "foo",
            "foo",
            vec![
                types::RecordElement::new("x", types::None::new(Position::dummy())),
                types::RecordElement::new("y", types::None::new(Position::dummy())),
            ],
            false,
            false,
            false,
            Position::dummy(),
        );
        let record_type = types::Record::new(type_definition.name(), Position::dummy());

        assert_eq!(
            transform_module(&Module::empty().set_type_definitions(vec![type_definition.clone()])),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition.clone()])
                .set_definitions(vec![Definition::new(
                    "foo.$equal",
                    "foo.$equal",
                    Lambda::new(
                        vec![
                            Argument::new(LHS_NAME, record_type.clone()),
                            Argument::new(RHS_NAME, record_type.clone()),
                        ],
                        types::Boolean::new(Position::dummy()),
                        If::new(
                            EqualityOperation::new(
                                Some(types::None::new(Position::dummy()).into()),
                                EqualityOperator::Equal,
                                RecordDeconstruction::new(
                                    Some(record_type.clone().into()),
                                    Variable::new(LHS_NAME, Position::dummy()),
                                    "x",
                                    Position::dummy(),
                                ),
                                RecordDeconstruction::new(
                                    Some(record_type.clone().into()),
                                    Variable::new(RHS_NAME, Position::dummy()),
                                    "x",
                                    Position::dummy(),
                                ),
                                Position::dummy(),
                            ),
                            If::new(
                                EqualityOperation::new(
                                    Some(types::None::new(Position::dummy()).into()),
                                    EqualityOperator::Equal,
                                    RecordDeconstruction::new(
                                        Some(record_type.clone().into()),
                                        Variable::new(LHS_NAME, Position::dummy()),
                                        "y",
                                        Position::dummy(),
                                    ),
                                    RecordDeconstruction::new(
                                        Some(record_type.clone().into()),
                                        Variable::new(RHS_NAME, Position::dummy()),
                                        "y",
                                        Position::dummy(),
                                    ),
                                    Position::dummy(),
                                ),
                                Boolean::new(true, Position::dummy()),
                                Boolean::new(false, Position::dummy()),
                                Position::dummy(),
                            ),
                            Boolean::new(false, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                    false,
                    Position::dummy()
                )]))
        );
    }
}
