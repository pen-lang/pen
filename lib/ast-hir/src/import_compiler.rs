use super::name_qualifier;
use hir::{
    analysis::ir::{type_transformer, variable_renamer},
    ir,
    types::{self, Type},
};
use itertools::Itertools;
use lang::ast;
use std::collections::HashMap;

pub fn compile(
    module: &ir::Module,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
    prelude_module_interfaces: &[interface::Module],
) -> ir::Module {
    let module = compile_imports(
        module,
        &module_interfaces
            .values()
            .chain(prelude_module_interfaces)
            .collect::<Vec<_>>(),
    );

    let module = rename_types(&module, module_interfaces, prelude_module_interfaces);
    rename_variables(&module, module_interfaces, prelude_module_interfaces)
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
                            definition.elements().to_vec(),
                            definition.is_open(),
                            definition.is_public(),
                            true,
                            definition.position().clone(),
                        )
                    })
            })
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
            .unique_by(|alias| alias.name().to_string())
            .chain(module.type_aliases().iter().cloned())
            .collect(),
        module.foreign_declarations().to_vec(),
        module_interfaces
            .iter()
            .flat_map(|interface| interface.declarations())
            .map(|declaration| {
                ir::Declaration::new(
                    declaration.name(),
                    declaration.type_().clone(),
                    declaration.position().clone(),
                )
            })
            .unique_by(|declaration| declaration.name().to_string())
            .chain(module.declarations().iter().cloned())
            .collect(),
        module.definitions().to_vec(),
        module.position().clone(),
    )
}

fn rename_variables(
    module: &ir::Module,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
    prelude_module_interfaces: &[interface::Module],
) -> ir::Module {
    variable_renamer::rename(
        module,
        &module_interfaces
            .iter()
            .flat_map(|(path, module)| {
                module
                    .declarations()
                    .iter()
                    .map(|declaration| {
                        (
                            name_qualifier::qualify(path, declaration.original_name()),
                            declaration.name().into(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .chain(prelude_module_interfaces.iter().flat_map(|module| {
                module.declarations().iter().map(|declaration| {
                    (
                        declaration.original_name().into(),
                        declaration.name().into(),
                    )
                })
            }))
            .collect(),
    )
}

fn rename_types(
    module: &ir::Module,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
    prelude_module_interfaces: &[interface::Module],
) -> ir::Module {
    let names = module_interfaces
        .iter()
        .flat_map(|(path, module)| {
            module
                .type_definitions()
                .iter()
                .filter_map(|definition| {
                    if definition.is_public() {
                        Some((
                            name_qualifier::qualify(path, definition.original_name()),
                            definition.name().into(),
                        ))
                    } else {
                        None
                    }
                })
                .chain(module.type_aliases().iter().filter_map(|alias| {
                    if alias.is_public() {
                        Some((
                            name_qualifier::qualify(path, alias.original_name()),
                            alias.name().into(),
                        ))
                    } else {
                        None
                    }
                }))
                .collect::<Vec<_>>()
        })
        .chain(prelude_module_interfaces.iter().flat_map(|module| {
            module
                .type_definitions()
                .iter()
                .map(|definition| (definition.original_name().into(), definition.name().into()))
                .chain(
                    module
                        .type_aliases()
                        .iter()
                        .map(|alias| (alias.original_name().into(), alias.name().into())),
                )
        }))
        .collect::<HashMap<String, String>>();

    type_transformer::transform(module, |type_| match type_ {
        Type::Record(record) => types::Record::new(
            names
                .get(record.name())
                .cloned()
                .unwrap_or_else(|| record.name().into()),
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
    use crate::test;
    use hir::{
        test::{DefinitionFake, ModuleFake, TypeDefinitionFake},
        types,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn compile_empty_module() {
        assert_eq!(
            compile(&ir::Module::empty(), &Default::default(), &[]),
            ir::Module::empty()
        );
    }

    #[test]
    fn rename_variable() {
        assert_eq!(
            compile(
                &ir::Module::empty().set_definitions(vec![ir::Definition::fake(
                    "Foo",
                    ir::Lambda::new(
                        vec![],
                        types::None::new(test::position()),
                        ir::Variable::new("Bar'Bar", test::position()),
                        test::position(),
                    ),
                    true,
                )]),
                &vec![(
                    ast::InternalModulePath::new(vec!["Bar".into()]).into(),
                    interface::Module::new(
                        vec![],
                        vec![],
                        vec![interface::Declaration::new(
                            "RealBar",
                            "Bar",
                            types::Function::new(
                                vec![],
                                types::None::new(test::position()),
                                test::position()
                            ),
                            test::position()
                        )]
                    )
                )]
                .into_iter()
                .collect(),
                &[]
            ),
            ir::Module::empty()
                .set_declarations(vec![ir::Declaration::new(
                    "RealBar",
                    types::Function::new(
                        vec![],
                        types::None::new(test::position()),
                        test::position()
                    ),
                    test::position()
                )])
                .set_definitions(vec![ir::Definition::fake(
                    "Foo",
                    ir::Lambda::new(
                        vec![],
                        types::None::new(test::position()),
                        ir::Variable::new("RealBar", test::position()),
                        test::position(),
                    ),
                    true,
                )])
        );
    }

    #[test]
    fn rename_type_definition() {
        assert_eq!(
            compile(
                &ir::Module::empty()
                    .set_type_definitions(vec![ir::TypeDefinition::fake(
                        "Foo",
                        vec![types::RecordElement::new(
                            "foo",
                            types::Reference::new("Bar'Bar", test::position())
                        )],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![ir::Definition::fake(
                        "Foo",
                        ir::Lambda::new(
                            vec![],
                            types::Reference::new("Bar'Bar", test::position()),
                            ir::None::new(test::position()),
                            test::position(),
                        ),
                        true,
                    )]),
                &vec![(
                    ast::InternalModulePath::new(vec!["Bar".into()]).into(),
                    interface::Module::new(
                        vec![interface::TypeDefinition::new(
                            "RealBar",
                            "Bar",
                            vec![],
                            false,
                            true,
                            test::position()
                        )],
                        vec![],
                        vec![]
                    )
                )]
                .into_iter()
                .collect(),
                &[]
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
                        test::position()
                    ),
                    ir::TypeDefinition::fake(
                        "Foo",
                        vec![types::RecordElement::new(
                            "foo",
                            types::Reference::new("RealBar", test::position())
                        )],
                        false,
                        false,
                        false,
                    )
                ])
                .set_definitions(vec![ir::Definition::fake(
                    "Foo",
                    ir::Lambda::new(
                        vec![],
                        types::Reference::new("RealBar", test::position()),
                        ir::None::new(test::position()),
                        test::position(),
                    ),
                    true,
                )])
        );
    }

    #[test]
    fn rename_type_alias() {
        assert_eq!(
            compile(
                &ir::Module::empty()
                    .set_type_definitions(vec![ir::TypeDefinition::fake(
                        "Foo",
                        vec![types::RecordElement::new(
                            "foo",
                            types::Reference::new("Bar'Bar", test::position())
                        )],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![ir::Definition::fake(
                        "Foo",
                        ir::Lambda::new(
                            vec![],
                            types::Reference::new("Bar'Bar", test::position()),
                            ir::None::new(test::position()),
                            test::position(),
                        ),
                        true,
                    )]),
                &vec![(
                    ast::InternalModulePath::new(vec!["Bar".into()]).into(),
                    interface::Module::new(
                        vec![],
                        vec![interface::TypeAlias::new(
                            "RealBar",
                            "Bar",
                            types::None::new(test::position()),
                            true,
                            test::position(),
                        )],
                        vec![]
                    )
                )]
                .into_iter()
                .collect(),
                &[]
            ),
            ir::Module::empty()
                .set_type_definitions(vec![ir::TypeDefinition::fake(
                    "Foo",
                    vec![types::RecordElement::new(
                        "foo",
                        types::Reference::new("RealBar", test::position())
                    )],
                    false,
                    false,
                    false,
                )])
                .set_type_aliases(vec![ir::TypeAlias::new(
                    "RealBar",
                    "Bar",
                    types::None::new(test::position()),
                    true,
                    true,
                    test::position(),
                )])
                .set_definitions(vec![ir::Definition::fake(
                    "Foo",
                    ir::Lambda::new(
                        vec![],
                        types::Reference::new("RealBar", test::position()),
                        ir::None::new(test::position()),
                        test::position(),
                    ),
                    true,
                )])
        );
    }

    #[test]
    fn do_not_rename_private_type_definition() {
        let type_definition = ir::TypeDefinition::fake(
            "Foo",
            vec![types::RecordElement::new(
                "foo",
                types::Reference::new("Bar'Bar", test::position()),
            )],
            false,
            false,
            false,
        );
        let definition = ir::Definition::fake(
            "Foo",
            ir::Lambda::new(
                vec![],
                types::Reference::new("Bar'Bar", test::position()),
                ir::None::new(test::position()),
                test::position(),
            ),
            true,
        );

        assert_eq!(
            compile(
                &ir::Module::empty()
                    .set_type_definitions(vec![type_definition.clone()])
                    .set_definitions(vec![definition.clone()]),
                &vec![(
                    ast::InternalModulePath::new(vec!["Bar".into()]).into(),
                    interface::Module::new(
                        vec![interface::TypeDefinition::new(
                            "RealBar",
                            "Bar",
                            vec![],
                            false,
                            false,
                            test::position()
                        )],
                        vec![],
                        vec![],
                    )
                )]
                .into_iter()
                .collect(),
                &[]
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
                        test::position()
                    ),
                    type_definition
                ])
                .set_definitions(vec![definition])
        );
    }

    #[test]
    fn do_not_rename_private_type_alias() {
        let type_definition = ir::TypeDefinition::fake(
            "Foo",
            vec![types::RecordElement::new(
                "foo",
                types::Reference::new("Bar'Bar", test::position()),
            )],
            false,
            false,
            false,
        );
        let definition = ir::Definition::fake(
            "Foo",
            ir::Lambda::new(
                vec![],
                types::Reference::new("Bar'Bar", test::position()),
                ir::None::new(test::position()),
                test::position(),
            ),
            true,
        );

        assert_eq!(
            compile(
                &ir::Module::empty()
                    .set_type_definitions(vec![type_definition.clone()])
                    .set_definitions(vec![definition.clone()]),
                &vec![(
                    ast::InternalModulePath::new(vec!["Bar".into()]).into(),
                    interface::Module::new(
                        vec![],
                        vec![interface::TypeAlias::new(
                            "RealBar",
                            "Bar",
                            types::None::new(test::position()),
                            false,
                            test::position(),
                        )],
                        vec![]
                    )
                )]
                .into_iter()
                .collect(),
                &[],
            ),
            ir::Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_type_aliases(vec![ir::TypeAlias::new(
                    "RealBar",
                    "Bar",
                    types::None::new(test::position()),
                    false,
                    true,
                    test::position()
                )])
                .set_definitions(vec![definition])
        );
    }
}
