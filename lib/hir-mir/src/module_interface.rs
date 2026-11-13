use super::error::CompileError;
use hir::{analysis::type_extractor, ir};

pub fn compile(module: &ir::Module) -> Result<interface::Module, CompileError> {
    Ok(interface::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|definition| {
                interface::TypeDefinition::new(
                    definition.name(),
                    definition.original_name(),
                    definition.fields().to_vec(),
                    definition.is_open(),
                    definition.is_public() && !definition.is_external(),
                    definition.position().clone(),
                )
            })
            .collect(),
        module
            .type_aliases()
            .iter()
            .map(|alias| {
                interface::TypeAlias::new(
                    alias.name(),
                    alias.original_name(),
                    alias.type_().clone(),
                    alias.is_public() && !alias.is_external(),
                    alias.position().clone(),
                )
            })
            .collect(),
        module
            .function_definitions()
            .iter()
            .filter_map(|definition| {
                if definition.is_public() {
                    Some(interface::FunctionDeclaration::new(
                        definition.name(),
                        definition.original_name(),
                        type_extractor::extract_from_lambda(definition.lambda()),
                        definition.position().clone(),
                    ))
                } else {
                    None
                }
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::{
        test::{FunctionDefinitionFake, ModuleFake},
        types,
    };
    use position::{Position, test::PositionFake};

    #[test]
    fn compile_empty_module() {
        assert_eq!(
            compile(&ir::Module::empty()),
            Ok(interface::Module::new(vec![], vec![], vec![]))
        );
    }

    #[test]
    fn compile_without_private_declaration() {
        assert_eq!(
            compile(&ir::Module::empty().set_function_definitions(vec![
                ir::FunctionDefinition::fake(
                    "foo",
                    ir::Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        ir::None::new(Position::fake()),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            Ok(interface::Module::new(vec![], vec![], vec![]))
        );
    }
}
