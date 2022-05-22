use super::TypeCheckError;
use crate::ir::*;
use fnv::FnvHashSet;

pub fn check_names(module: &Module) -> Result<(), TypeCheckError> {
    check_types(module.type_definitions())?;
    check_functions(module)?;

    Ok(())
}

fn check_types(definitions: &[TypeDefinition]) -> Result<(), TypeCheckError> {
    let mut names = FnvHashSet::default();

    for definition in definitions {
        if names.contains(definition.name()) {
            return Err(TypeCheckError::DuplicateTypeNames(definition.name().into()));
        }

        names.insert(definition.name());
    }

    Ok(())
}

fn check_functions(module: &Module) -> Result<(), TypeCheckError> {
    let mut names = FnvHashSet::default();

    for name in module
        .foreign_declarations()
        .iter()
        .map(|declaration| declaration.name())
        .chain(
            module
                .function_declarations()
                .iter()
                .map(|declaration| declaration.name()),
        )
        .chain(
            module
                .function_definitions()
                .iter()
                .map(|definition| definition.name()),
        )
    {
        if names.contains(name) {
            return Err(TypeCheckError::DuplicateFunctionNames(name.into()));
        }

        names.insert(name);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{self, Type};

    #[test]
    fn check_with_empty_modules() {
        assert_eq!(
            check_names(&Module::new(vec![], vec![], vec![], vec![], vec![])),
            Ok(())
        );
    }

    #[test]
    fn check_duplicate_type_name() {
        let module = Module::new(
            vec![
                TypeDefinition::new("Foo", types::RecordBody::new(vec![])),
                TypeDefinition::new("Foo", types::RecordBody::new(vec![])),
            ],
            vec![],
            vec![],
            vec![],
            vec![],
        );

        assert_eq!(
            check_names(&module),
            Err(TypeCheckError::DuplicateTypeNames("Foo".into()))
        );
    }

    #[test]
    fn check_duplicate_function_name_in_definition() {
        let module = Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Variable::new("x"),
                    Type::Number,
                ),
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Variable::new("x"),
                    Type::Number,
                ),
            ],
        );

        assert_eq!(
            check_names(&module),
            Err(TypeCheckError::DuplicateFunctionNames("f".into()))
        );
    }

    #[test]
    fn check_duplicate_function_name_in_foreign_declaration() {
        let module = Module::new(
            vec![],
            vec![ForeignDeclaration::new(
                "f",
                "g",
                types::Function::new(vec![Type::Number], Type::Number),
                CallingConvention::Target,
            )],
            vec![],
            vec![],
            vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                Variable::new("x"),
                Type::Number,
            )],
        );

        assert_eq!(
            check_names(&module),
            Err(TypeCheckError::DuplicateFunctionNames("f".into()))
        );
    }

    #[test]
    fn check_duplicate_function_name_in_declaration() {
        let module = Module::new(
            vec![],
            vec![],
            vec![],
            vec![FunctionDeclaration::new(
                "f",
                types::Function::new(vec![Type::Number], Type::Number),
            )],
            vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                Variable::new("x"),
                Type::Number,
            )],
        );

        assert_eq!(
            check_names(&module),
            Err(TypeCheckError::DuplicateFunctionNames("f".into()))
        );
    }
}
