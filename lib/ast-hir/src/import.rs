use super::name;
use crate::imported_module::ImportedModule;
use fnv::FnvHashMap;
use hir::{
    analysis::{type_transformer, variable_renamer},
    ir,
    types::{self, Type},
};
use itertools::Itertools;

pub fn compile(
    module: &ir::Module,
    imported_modules: &[ImportedModule],
    prelude_module_interfaces: &[interface::Module],
) -> ir::Module {
    let module = rename_types(module, imported_modules);
    let module = rename_variables(&module, imported_modules);

    compile_imports(
        &module,
        &imported_modules
            .iter()
            .map(|module| module.interface())
            .chain(prelude_module_interfaces)
            .collect::<Vec<_>>(),
    )
}

fn compile_imports(module: &ir::Module, module_interfaces: &[&interface::Module]) -> ir::Module {
    ir::Module::new(
        module_interfaces
            .iter()
            .flat_map(|module_interface| {
                module_interface
                    .type_definitions()
                    .iter()
                    .map(|definition| {
                        ir::TypeDefinition::new(
                            definition.name(),
                            definition.original_name(),
                            definition.fields().to_vec(),
                            definition.is_open(),
                            definition.is_public(),
                            true,
                            definition.position().clone(),
                        )
                    })
            })
            .sorted_by_key(|definition| (definition.name().to_string(), !definition.is_public()))
            .unique_by(|definition| definition.name().to_string())
            .chain(module.type_definitions().iter().cloned())
            .collect(),
        module_interfaces
            .iter()
            .flat_map(|module_interface| {
                module_interface.type_aliases().iter().map(|alias| {
                    ir::TypeAlias::new(
                        alias.name(),
                        alias.original_name(),
                        alias.type_().clone(),
                        alias.is_public(),
                        true,
                        alias.position().clone(),
                    )
                })
            })
            .sorted_by_key(|alias| (alias.name().to_string(), !alias.is_public()))
            .unique_by(|alias| alias.name().to_string())
            .chain(module.type_aliases().iter().cloned())
            .collect(),
        module.foreign_declarations().to_vec(),
        module_interfaces
            .iter()
            .flat_map(|interface| interface.function_declarations())
            .map(|declaration| {
                ir::FunctionDeclaration::new(
                    declaration.name(),
                    declaration.type_().clone(),
                    declaration.position().clone(),
                )
            })
            .unique_by(|declaration| declaration.name().to_string())
            .chain(module.function_declarations().iter().cloned())
            .collect(),
        module.function_definitions().to_vec(),
        module.position().clone(),
    )
}

fn rename_variables(module: &ir::Module, imported_modules: &[ImportedModule]) -> ir::Module {
    let mut names = imported_modules
        .iter()
        .flat_map(|module| {
            module
                .interface()
                .function_declarations()
                .iter()
                .map(|declaration| {
                    (
                        if module
                            .unqualified_names()
                            .contains_key(declaration.original_name())
                        {
                            declaration.original_name().into()
                        } else {
                            name::qualify(module.prefix(), declaration.original_name())
                        },
                        declaration.name().into(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<FnvHashMap<_, _>>();

    for definition in module.function_definitions() {
        names.remove(definition.original_name());
    }

    variable_renamer::rename(module, &names)
}

fn rename_types(module: &ir::Module, imported_modules: &[ImportedModule]) -> ir::Module {
    let mut names = imported_modules
        .iter()
        .flat_map(|module| {
            module
                .interface()
                .type_definitions()
                .iter()
                .filter(|definition| definition.is_public())
                .map(|definition| (definition.original_name(), definition.name()))
                .chain(
                    module
                        .interface()
                        .type_aliases()
                        .iter()
                        .filter(|alias| alias.is_public())
                        .map(|alias| (alias.original_name(), alias.name())),
                )
                .map(|(original_name, name)| {
                    (
                        if module.unqualified_names().contains_key(original_name) {
                            original_name.into()
                        } else {
                            name::qualify(module.prefix(), original_name)
                        },
                        name.into(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<FnvHashMap<_, String>>();

    for definition in module.type_definitions() {
        names.remove(definition.original_name());
    }

    for alias in module.type_aliases() {
        names.remove(alias.original_name());
    }

    type_transformer::transform(module, |type_| match type_ {
        Type::Record(record) => types::Record::new(
            names
                .get(record.name())
                .cloned()
                .unwrap_or_else(|| record.name().into()),
            record.original_name(),
            record.position().clone(),
        )
        .into(),
        Type::Reference(reference) => types::Reference::new(
            names
                .get(reference.name())
                .cloned()
                .unwrap_or_else(|| reference.name().into()),
            reference.position().clone(),
        )
        .into(),
        _ => type_.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::{
        test::{FunctionDefinitionFake, ModuleFake, TypeAliasFake, TypeDefinitionFake},
        types,
    };
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    fn compile_module(
        module: &ir::Module,
        explicitly_imported_modules: &[ImportedModule],
    ) -> ir::Module {
        compile(module, explicitly_imported_modules, &[])
    }

    #[test]
    fn compile_empty_module() {
        assert_eq!(
            compile_module(&ir::Module::empty(), &[]),
            ir::Module::empty()
        );
    }

    mod function_definition {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn rename_variable() {
            assert_eq!(
                compile_module(
                    &ir::Module::empty().set_function_definitions(vec![
                        ir::FunctionDefinition::fake(
                            "Foo",
                            ir::Lambda::new(
                                vec![],
                                types::None::new(Position::fake()),
                                ir::Variable::new("Bar'Bar", Position::fake()),
                                Position::fake(),
                            ),
                            true,
                        )
                    ]),
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![],
                            vec![],
                            vec![interface::FunctionDeclaration::new(
                                "RealBar",
                                "Bar",
                                types::Function::new(
                                    vec![],
                                    types::None::new(Position::fake()),
                                    Position::fake()
                                ),
                                Position::fake()
                            )]
                        ),
                        "Bar",
                        Default::default(),
                    )],
                ),
                ir::Module::empty()
                    .set_function_declarations(vec![ir::FunctionDeclaration::new(
                        "RealBar",
                        types::Function::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Position::fake()
                        ),
                        Position::fake()
                    )])
                    .set_function_definitions(vec![ir::FunctionDefinition::fake(
                        "Foo",
                        ir::Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            ir::Variable::new("RealBar", Position::fake()),
                            Position::fake(),
                        ),
                        true,
                    )])
            );
        }

        #[test]
        fn do_not_rename_overridden_variable() {
            let module =
                ir::Module::empty().set_function_definitions(vec![ir::FunctionDefinition::new(
                    "Foo.Foo",
                    "Foo",
                    ir::Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        ir::Variable::new("Foo", Position::fake()),
                        Position::fake(),
                    ),
                    None,
                    true,
                    Position::fake(),
                )]);

            assert_eq!(
                compile_module(
                    &module,
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![],
                            vec![],
                            vec![interface::FunctionDeclaration::new(
                                "Bar.Foo",
                                "Foo",
                                types::Function::new(
                                    vec![],
                                    types::None::new(Position::fake()),
                                    Position::fake()
                                ),
                                Position::fake()
                            )]
                        ),
                        "Bar",
                        [("Foo".into(), Position::fake())].into_iter().collect(),
                    )],
                ),
                module.set_function_declarations(vec![ir::FunctionDeclaration::new(
                    "Bar.Foo",
                    types::Function::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
                )])
            );
        }
    }

    mod type_definition {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn rename_type_definition() {
            assert_eq!(
                compile_module(
                    &ir::Module::empty()
                        .set_type_definitions(vec![ir::TypeDefinition::fake(
                            "Foo",
                            vec![types::RecordField::new(
                                "foo",
                                types::Reference::new("Bar'Bar", Position::fake())
                            )],
                            false,
                            false,
                            false,
                        )])
                        .set_function_definitions(vec![ir::FunctionDefinition::fake(
                            "Foo",
                            ir::Lambda::new(
                                vec![],
                                types::Reference::new("Bar'Bar", Position::fake()),
                                ir::None::new(Position::fake()),
                                Position::fake(),
                            ),
                            true,
                        )]),
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![interface::TypeDefinition::new(
                                "RealBar",
                                "Bar",
                                vec![],
                                false,
                                true,
                                Position::fake()
                            )],
                            vec![],
                            vec![]
                        ),
                        "Bar",
                        Default::default()
                    )],
                ),
                ir::Module::empty()
                    .set_type_definitions(vec![
                        ir::TypeDefinition::new(
                            "RealBar",
                            "Bar",
                            vec![],
                            false,
                            true,
                            true,
                            Position::fake()
                        ),
                        ir::TypeDefinition::fake(
                            "Foo",
                            vec![types::RecordField::new(
                                "foo",
                                types::Reference::new("RealBar", Position::fake())
                            )],
                            false,
                            false,
                            false,
                        )
                    ])
                    .set_function_definitions(vec![ir::FunctionDefinition::fake(
                        "Foo",
                        ir::Lambda::new(
                            vec![],
                            types::Reference::new("RealBar", Position::fake()),
                            ir::None::new(Position::fake()),
                            Position::fake(),
                        ),
                        true,
                    )])
            );
        }

        #[test]
        fn do_not_rename_private_type_definition() {
            let type_definition = ir::TypeDefinition::fake(
                "Foo",
                vec![types::RecordField::new(
                    "foo",
                    types::Reference::new("Bar'Bar", Position::fake()),
                )],
                false,
                false,
                false,
            );
            let definition = ir::FunctionDefinition::fake(
                "Foo",
                ir::Lambda::new(
                    vec![],
                    types::Reference::new("Bar'Bar", Position::fake()),
                    ir::None::new(Position::fake()),
                    Position::fake(),
                ),
                true,
            );

            assert_eq!(
                compile_module(
                    &ir::Module::empty()
                        .set_type_definitions(vec![type_definition.clone()])
                        .set_function_definitions(vec![definition.clone()]),
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![interface::TypeDefinition::new(
                                "RealBar",
                                "Bar",
                                vec![],
                                false,
                                false,
                                Position::fake()
                            )],
                            vec![],
                            vec![],
                        ),
                        "Bar",
                        Default::default()
                    )],
                ),
                ir::Module::empty()
                    .set_type_definitions(vec![
                        ir::TypeDefinition::new(
                            "RealBar",
                            "Bar",
                            vec![],
                            false,
                            false,
                            true,
                            Position::fake()
                        ),
                        type_definition
                    ])
                    .set_function_definitions(vec![definition])
            );
        }

        #[test]
        fn prefer_public_definition_of_same_type_definition() {
            let create_type_definition = |public| {
                interface::TypeDefinition::new(
                    "Foo",
                    "Foo",
                    vec![],
                    false,
                    public,
                    Position::fake(),
                )
            };

            assert_eq!(
                compile_module(
                    &ir::Module::empty(),
                    &vec![
                        ImportedModule::new(
                            interface::Module::new(
                                vec![create_type_definition(false)],
                                vec![],
                                vec![]
                            ),
                            "Foo",
                            Default::default()
                        ),
                        ImportedModule::new(
                            interface::Module::new(
                                vec![create_type_definition(true)],
                                vec![],
                                vec![]
                            ),
                            "Bar",
                            Default::default()
                        )
                    ],
                ),
                ir::Module::empty().set_type_definitions(vec![ir::TypeDefinition::fake(
                    "Foo",
                    vec![],
                    false,
                    true,
                    true,
                )])
            );
        }

        #[test]
        fn do_not_rename_overridden_type_definition() {
            let module = ir::Module::empty()
                .set_type_aliases(vec![ir::TypeAlias::new(
                    "Foo.Foo",
                    "Foo",
                    types::Reference::new("Bar", Position::fake()),
                    false,
                    false,
                    Position::fake(),
                )])
                .set_type_definitions(vec![ir::TypeDefinition::new(
                    "Foo.Bar",
                    "Bar",
                    vec![],
                    false,
                    false,
                    false,
                    Position::fake(),
                )]);

            assert_eq!(
                compile_module(
                    &module,
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![interface::TypeDefinition::new(
                                "Bar.Bar",
                                "Bar",
                                vec![],
                                false,
                                true,
                                Position::fake()
                            )],
                            vec![],
                            vec![]
                        ),
                        "Bar",
                        [("Bar".into(), Position::fake())].into_iter().collect(),
                    )],
                ),
                module.set_type_definitions(
                    [ir::TypeDefinition::new(
                        "Bar.Bar",
                        "Bar",
                        vec![],
                        false,
                        true,
                        true,
                        Position::fake(),
                    )]
                    .into_iter()
                    .chain(module.type_definitions().iter().cloned())
                    .collect()
                )
            );
        }
    }

    mod type_alias {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn rename_type_alias() {
            assert_eq!(
                compile_module(
                    &ir::Module::empty()
                        .set_type_definitions(vec![ir::TypeDefinition::fake(
                            "Foo",
                            vec![types::RecordField::new(
                                "foo",
                                types::Reference::new("Bar'Bar", Position::fake())
                            )],
                            false,
                            false,
                            false,
                        )])
                        .set_function_definitions(vec![ir::FunctionDefinition::fake(
                            "Foo",
                            ir::Lambda::new(
                                vec![],
                                types::Reference::new("Bar'Bar", Position::fake()),
                                ir::None::new(Position::fake()),
                                Position::fake(),
                            ),
                            true,
                        )]),
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![],
                            vec![interface::TypeAlias::new(
                                "RealBar",
                                "Bar",
                                types::None::new(Position::fake()),
                                true,
                                Position::fake(),
                            )],
                            vec![]
                        ),
                        "Bar",
                        Default::default()
                    )],
                ),
                ir::Module::empty()
                    .set_type_definitions(vec![ir::TypeDefinition::fake(
                        "Foo",
                        vec![types::RecordField::new(
                            "foo",
                            types::Reference::new("RealBar", Position::fake())
                        )],
                        false,
                        false,
                        false,
                    )])
                    .set_type_aliases(vec![ir::TypeAlias::new(
                        "RealBar",
                        "Bar",
                        types::None::new(Position::fake()),
                        true,
                        true,
                        Position::fake(),
                    )])
                    .set_function_definitions(vec![ir::FunctionDefinition::fake(
                        "Foo",
                        ir::Lambda::new(
                            vec![],
                            types::Reference::new("RealBar", Position::fake()),
                            ir::None::new(Position::fake()),
                            Position::fake(),
                        ),
                        true,
                    )])
            );
        }

        #[test]
        fn do_not_rename_private_type_alias() {
            let type_definition = ir::TypeDefinition::fake(
                "Foo",
                vec![types::RecordField::new(
                    "foo",
                    types::Reference::new("Bar'Bar", Position::fake()),
                )],
                false,
                false,
                false,
            );
            let definition = ir::FunctionDefinition::fake(
                "Foo",
                ir::Lambda::new(
                    vec![],
                    types::Reference::new("Bar'Bar", Position::fake()),
                    ir::None::new(Position::fake()),
                    Position::fake(),
                ),
                true,
            );

            assert_eq!(
                compile_module(
                    &ir::Module::empty()
                        .set_type_definitions(vec![type_definition.clone()])
                        .set_function_definitions(vec![definition.clone()]),
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![],
                            vec![interface::TypeAlias::new(
                                "RealBar",
                                "Bar",
                                types::None::new(Position::fake()),
                                false,
                                Position::fake(),
                            )],
                            vec![]
                        ),
                        "Bar",
                        Default::default()
                    )],
                ),
                ir::Module::empty()
                    .set_type_definitions(vec![type_definition])
                    .set_type_aliases(vec![ir::TypeAlias::new(
                        "RealBar",
                        "Bar",
                        types::None::new(Position::fake()),
                        false,
                        true,
                        Position::fake()
                    )])
                    .set_function_definitions(vec![definition])
            );
        }

        #[test]
        fn prefer_public_definition_of_same_type_alias() {
            let create_type_alias = |public| {
                interface::TypeAlias::new(
                    "Foo",
                    "",
                    types::None::new(Position::fake()),
                    public,
                    Position::fake(),
                )
            };

            assert_eq!(
                compile_module(
                    &ir::Module::empty(),
                    &vec![
                        ImportedModule::new(
                            interface::Module::new(vec![], vec![create_type_alias(false)], vec![]),
                            "Foo",
                            Default::default()
                        ),
                        ImportedModule::new(
                            interface::Module::new(vec![], vec![create_type_alias(true)], vec![]),
                            "Bar",
                            Default::default()
                        )
                    ],
                ),
                ir::Module::empty().set_type_aliases(vec![ir::TypeAlias::fake(
                    "Foo",
                    types::None::new(Position::fake()),
                    true,
                    true,
                )])
            );
        }

        #[test]
        fn do_not_rename_overridden_type_alias() {
            let module = ir::Module::empty().set_type_aliases(vec![
                ir::TypeAlias::new(
                    "Foo.Foo",
                    "Foo",
                    types::Reference::new("Bar", Position::fake()),
                    false,
                    false,
                    Position::fake(),
                ),
                ir::TypeAlias::new(
                    "Foo.Bar",
                    "Bar",
                    types::Reference::new("Baz", Position::fake()),
                    false,
                    false,
                    Position::fake(),
                ),
            ]);

            assert_eq!(
                compile_module(
                    &module,
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![interface::TypeDefinition::new(
                                "Bar.Bar",
                                "Bar",
                                vec![],
                                false,
                                true,
                                Position::fake()
                            )],
                            vec![],
                            vec![]
                        ),
                        "Bar",
                        [("Bar".into(), Position::fake())].into_iter().collect(),
                    )],
                ),
                module.set_type_definitions(vec![ir::TypeDefinition::new(
                    "Bar.Bar",
                    "Bar",
                    vec![],
                    false,
                    true,
                    true,
                    Position::fake(),
                )])
            );
        }
    }

    mod unqualified_import {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn rename_variable() {
            assert_eq!(
                compile_module(
                    &ir::Module::empty().set_function_definitions(vec![
                        ir::FunctionDefinition::fake(
                            "Foo",
                            ir::Lambda::new(
                                vec![],
                                types::None::new(Position::fake()),
                                ir::Variable::new("Bar", Position::fake()),
                                Position::fake(),
                            ),
                            true,
                        )
                    ]),
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![],
                            vec![],
                            vec![interface::FunctionDeclaration::new(
                                "RealBar",
                                "Bar",
                                types::Function::new(
                                    vec![],
                                    types::None::new(Position::fake()),
                                    Position::fake()
                                ),
                                Position::fake()
                            )]
                        ),
                        "Bar",
                        [("Bar".into(), Position::fake())].into_iter().collect()
                    )],
                ),
                ir::Module::empty()
                    .set_function_declarations(vec![ir::FunctionDeclaration::new(
                        "RealBar",
                        types::Function::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Position::fake()
                        ),
                        Position::fake()
                    )])
                    .set_function_definitions(vec![ir::FunctionDefinition::fake(
                        "Foo",
                        ir::Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            ir::Variable::new("RealBar", Position::fake()),
                            Position::fake(),
                        ),
                        true,
                    )])
            );
        }

        #[test]
        fn rename_type_definition() {
            assert_eq!(
                compile_module(
                    &ir::Module::empty()
                        .set_type_definitions(vec![ir::TypeDefinition::fake(
                            "Foo",
                            vec![types::RecordField::new(
                                "foo",
                                types::Reference::new("Bar", Position::fake())
                            )],
                            false,
                            false,
                            false,
                        )])
                        .set_function_definitions(vec![ir::FunctionDefinition::fake(
                            "Foo",
                            ir::Lambda::new(
                                vec![],
                                types::Reference::new("Bar", Position::fake()),
                                ir::None::new(Position::fake()),
                                Position::fake(),
                            ),
                            true,
                        )]),
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![interface::TypeDefinition::new(
                                "RealBar",
                                "Bar",
                                vec![],
                                false,
                                true,
                                Position::fake()
                            )],
                            vec![],
                            vec![]
                        ),
                        "Bar",
                        [("Bar".into(), Position::fake())].into_iter().collect()
                    )],
                ),
                ir::Module::empty()
                    .set_type_definitions(vec![
                        ir::TypeDefinition::new(
                            "RealBar",
                            "Bar",
                            vec![],
                            false,
                            true,
                            true,
                            Position::fake()
                        ),
                        ir::TypeDefinition::fake(
                            "Foo",
                            vec![types::RecordField::new(
                                "foo",
                                types::Reference::new("RealBar", Position::fake())
                            )],
                            false,
                            false,
                            false,
                        )
                    ])
                    .set_function_definitions(vec![ir::FunctionDefinition::fake(
                        "Foo",
                        ir::Lambda::new(
                            vec![],
                            types::Reference::new("RealBar", Position::fake()),
                            ir::None::new(Position::fake()),
                            Position::fake(),
                        ),
                        true,
                    )])
            );
        }

        #[test]
        fn rename_type_alias() {
            assert_eq!(
                compile_module(
                    &ir::Module::empty()
                        .set_type_definitions(vec![ir::TypeDefinition::fake(
                            "Foo",
                            vec![types::RecordField::new(
                                "foo",
                                types::Reference::new("Bar", Position::fake())
                            )],
                            false,
                            false,
                            false,
                        )])
                        .set_function_definitions(vec![ir::FunctionDefinition::fake(
                            "Foo",
                            ir::Lambda::new(
                                vec![],
                                types::Reference::new("Bar", Position::fake()),
                                ir::None::new(Position::fake()),
                                Position::fake(),
                            ),
                            true,
                        )]),
                    &[ImportedModule::new(
                        interface::Module::new(
                            vec![],
                            vec![interface::TypeAlias::new(
                                "RealBar",
                                "Bar",
                                types::None::new(Position::fake()),
                                true,
                                Position::fake(),
                            )],
                            vec![]
                        ),
                        "Bar",
                        [("Bar".into(), Position::fake())].into_iter().collect()
                    )],
                ),
                ir::Module::empty()
                    .set_type_definitions(vec![ir::TypeDefinition::fake(
                        "Foo",
                        vec![types::RecordField::new(
                            "foo",
                            types::Reference::new("RealBar", Position::fake())
                        )],
                        false,
                        false,
                        false,
                    )])
                    .set_type_aliases(vec![ir::TypeAlias::new(
                        "RealBar",
                        "Bar",
                        types::None::new(Position::fake()),
                        true,
                        true,
                        Position::fake(),
                    )])
                    .set_function_definitions(vec![ir::FunctionDefinition::fake(
                        "Foo",
                        ir::Lambda::new(
                            vec![],
                            types::Reference::new("RealBar", Position::fake()),
                            ir::None::new(Position::fake()),
                            Position::fake(),
                        ),
                        true,
                    )])
            );
        }
    }
}
