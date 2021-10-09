use super::name_qualifier;
use crate::imported_module::ImportedModule;
use hir::{analysis::ir::variable_transformer, ir::*, types};
use std::collections::HashMap;

pub fn compile(module: &Module, imported_modules: &[ImportedModule]) -> Module {
    let names = imported_modules
        .iter()
        .flat_map(|module| {
            module
                .interface()
                .type_definitions()
                .iter()
                .filter_map(|definition| {
                    if definition.fields().is_empty() && definition.is_public() {
                        Some((
                            name_qualifier::qualify(module.prefix(), definition.original_name()),
                            definition.name().into(),
                        ))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .chain(module.type_definitions().iter().filter_map(|definition| {
            if definition.fields().is_empty() && !definition.is_external() {
                Some((definition.original_name().into(), definition.name().into()))
            } else {
                None
            }
        }))
        .collect::<HashMap<String, String>>();

    variable_transformer::transform(module, &|variable| {
        if let Some(record_name) = names.get(variable.name()) {
            RecordConstruction::new(
                types::Record::new(record_name, variable.position().clone()),
                vec![],
                variable.position().clone(),
            )
            .into()
        } else {
            variable.clone().into()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::test::{DefinitionFake, ModuleFake};
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    #[test]
    fn compile_singleton_record() {
        let type_definition =
            TypeDefinition::new("bar", "foo", vec![], false, false, false, Position::fake());

        assert_eq!(
            compile(
                &Module::empty()
                    .set_type_definitions(vec![type_definition.clone()])
                    .set_definitions(vec![Definition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Variable::new("foo", Position::fake()),
                            Position::fake()
                        ),
                        false
                    )]),
                &[],
            ),
            Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        RecordConstruction::new(
                            types::Record::new("bar", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    false
                )])
        );
    }

    #[test]
    fn do_not_compile_non_singleton_record() {
        let type_definition = TypeDefinition::new(
            "bar",
            "foo",
            vec![types::RecordField::new(
                "x",
                types::None::new(Position::fake()),
            )],
            false,
            false,
            false,
            Position::fake(),
        );
        let definition = Definition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                Variable::new("foo", Position::fake()),
                Position::fake(),
            ),
            false,
        );

        assert_eq!(
            compile(
                &Module::empty()
                    .set_type_definitions(vec![type_definition.clone()])
                    .set_definitions(vec![definition.clone()]),
                &[],
            ),
            Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_definitions(vec![definition])
        );
    }

    #[test]
    fn compile_imported_singleton_record() {
        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Variable::new("Foo'Foo", Position::fake()),
                        Position::fake()
                    ),
                    false
                )]),
                &[ImportedModule::new(
                    interface::Module::new(
                        vec![interface::TypeDefinition::new(
                            "RealFoo",
                            "Foo",
                            vec![],
                            false,
                            true,
                            Position::fake()
                        )],
                        vec![],
                        vec![]
                    ),
                    "Foo",
                    Default::default()
                )],
            ),
            Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    RecordConstruction::new(
                        types::Record::new("RealFoo", Position::fake()),
                        vec![],
                        Position::fake()
                    ),
                    Position::fake()
                ),
                false
            )])
        );
    }

    #[test]
    fn do_not_compile_imported_non_singleton_record() {
        let definition = Definition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                Variable::new("Foo'Foo", Position::fake()),
                Position::fake(),
            ),
            false,
        );

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![definition.clone()]),
                &[ImportedModule::new(
                    interface::Module::new(
                        vec![interface::TypeDefinition::new(
                            "RealFoo",
                            "Foo",
                            vec![types::RecordField::new(
                                "x",
                                types::None::new(Position::fake()),
                            )],
                            false,
                            true,
                            Position::fake()
                        )],
                        vec![],
                        vec![]
                    ),
                    "Foo",
                    Default::default()
                )],
            ),
            Module::empty().set_definitions(vec![definition])
        );
    }

    #[test]
    fn do_not_compile_imported_private_record() {
        let definition = Definition::fake(
            "f",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                Variable::new("Foo'Foo", Position::fake()),
                Position::fake(),
            ),
            false,
        );

        assert_eq!(
            compile(
                &Module::empty().set_definitions(vec![definition.clone()]),
                &[ImportedModule::new(
                    interface::Module::new(
                        vec![interface::TypeDefinition::new(
                            "RealFoo",
                            "Foo",
                            vec![],
                            false,
                            false,
                            Position::fake()
                        )],
                        vec![],
                        vec![]
                    ),
                    "Foo",
                    Default::default()
                )],
            ),
            Module::empty().set_definitions(vec![definition])
        );
    }
}
