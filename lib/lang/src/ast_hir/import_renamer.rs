use super::utilities;
use crate::{
    ast,
    hir::{
        self,
        analysis::{type_transformer, variable_renamer},
    },
    interface,
    types::{self, Type},
};
use std::collections::HashMap;

pub fn rename(
    module: &hir::Module,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
    prelude_module_interfaces: &[interface::Module],
) -> hir::Module {
    let module = rename_types(module, module_interfaces, prelude_module_interfaces);
    rename_variables(&module, module_interfaces, prelude_module_interfaces)
}

fn rename_variables(
    module: &hir::Module,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
    prelude_module_interfaces: &[interface::Module],
) -> hir::Module {
    variable_renamer::rename(
        module,
        &module_interfaces
            .iter()
            .flat_map(|(path, module)| {
                let prefix = utilities::get_prefix(path);

                module
                    .declarations()
                    .iter()
                    .map(|declaration| {
                        (
                            utilities::qualify_name(prefix, declaration.original_name()),
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
    module: &hir::Module,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
    prelude_module_interfaces: &[interface::Module],
) -> hir::Module {
    let names = module_interfaces
        .iter()
        .flat_map(|(path, module)| {
            let prefix = utilities::get_prefix(path);

            module
                .type_definitions()
                .iter()
                .filter_map(|definition| {
                    if definition.is_public() {
                        Some((
                            utilities::qualify_name(prefix, definition.original_name()),
                            definition.name().into(),
                        ))
                    } else {
                        None
                    }
                })
                .chain(module.type_aliases().iter().filter_map(|alias| {
                    if alias.is_public() {
                        Some((
                            utilities::qualify_name(prefix, alias.original_name()),
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
    use crate::{position::Position, types};
    use pretty_assertions::assert_eq;

    #[test]
    fn rename_empty_module() {
        assert_eq!(
            rename(
                &hir::Module::new(vec![], vec![], vec![], vec![]),
                &Default::default(),
                &[]
            ),
            hir::Module::new(vec![], vec![], vec![], vec![])
        );
    }

    #[test]
    fn rename_variable() {
        assert_eq!(
            rename(
                &hir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![hir::Definition::new(
                        "Foo",
                        "Foo",
                        hir::Lambda::new(
                            vec![],
                            types::None::new(Position::dummy()),
                            hir::Variable::new("Bar'Bar", Position::dummy()),
                            Position::dummy(),
                        ),
                        true,
                        Position::dummy()
                    )]
                ),
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
                                types::None::new(Position::dummy()),
                                Position::dummy()
                            ),
                            Position::dummy()
                        )]
                    )
                )]
                .into_iter()
                .collect(),
                &[]
            ),
            hir::Module::new(
                vec![],
                vec![],
                vec![],
                vec![hir::Definition::new(
                    "Foo",
                    "Foo",
                    hir::Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        hir::Variable::new("RealBar", Position::dummy()),
                        Position::dummy(),
                    ),
                    true,
                    Position::dummy()
                )]
            )
        );
    }

    #[test]
    fn rename_type_definition() {
        assert_eq!(
            rename(
                &hir::Module::new(
                    vec![hir::TypeDefinition::without_source(
                        "Foo",
                        vec![types::RecordElement::new(
                            "foo",
                            types::Reference::new("Bar'Bar", Position::dummy())
                        )],
                        false,
                        false,
                        false,
                    )],
                    vec![],
                    vec![],
                    vec![hir::Definition::without_source(
                        "Foo",
                        hir::Lambda::new(
                            vec![],
                            types::Reference::new("Bar'Bar", Position::dummy()),
                            hir::None::new(Position::dummy()),
                            Position::dummy(),
                        ),
                        true,
                    )]
                ),
                &vec![(
                    ast::InternalModulePath::new(vec!["Bar".into()]).into(),
                    interface::Module::new(
                        vec![interface::TypeDefinition::new(
                            "RealBar",
                            "Bar",
                            vec![],
                            false,
                            true,
                            Position::dummy()
                        )],
                        vec![],
                        vec![]
                    )
                )]
                .into_iter()
                .collect(),
                &[]
            ),
            hir::Module::new(
                vec![hir::TypeDefinition::without_source(
                    "Foo",
                    vec![types::RecordElement::new(
                        "foo",
                        types::Reference::new("RealBar", Position::dummy())
                    )],
                    false,
                    false,
                    false,
                )],
                vec![],
                vec![],
                vec![hir::Definition::without_source(
                    "Foo",
                    hir::Lambda::new(
                        vec![],
                        types::Reference::new("RealBar", Position::dummy()),
                        hir::None::new(Position::dummy()),
                        Position::dummy(),
                    ),
                    true,
                )]
            )
        );
    }

    #[test]
    fn rename_type_alias() {
        assert_eq!(
            rename(
                &hir::Module::new(
                    vec![hir::TypeDefinition::without_source(
                        "Foo",
                        vec![types::RecordElement::new(
                            "foo",
                            types::Reference::new("Bar'Bar", Position::dummy())
                        )],
                        false,
                        false,
                        false,
                    )],
                    vec![],
                    vec![],
                    vec![hir::Definition::without_source(
                        "Foo",
                        hir::Lambda::new(
                            vec![],
                            types::Reference::new("Bar'Bar", Position::dummy()),
                            hir::None::new(Position::dummy()),
                            Position::dummy(),
                        ),
                        true,
                    )]
                ),
                &vec![(
                    ast::InternalModulePath::new(vec!["Bar".into()]).into(),
                    interface::Module::new(
                        vec![],
                        vec![interface::TypeAlias::new(
                            "RealBar",
                            "Bar",
                            types::None::new(Position::dummy()),
                            true,
                        )],
                        vec![]
                    )
                )]
                .into_iter()
                .collect(),
                &[]
            ),
            hir::Module::new(
                vec![hir::TypeDefinition::without_source(
                    "Foo",
                    vec![types::RecordElement::new(
                        "foo",
                        types::Reference::new("RealBar", Position::dummy())
                    )],
                    false,
                    false,
                    false,
                )],
                vec![],
                vec![],
                vec![hir::Definition::without_source(
                    "Foo",
                    hir::Lambda::new(
                        vec![],
                        types::Reference::new("RealBar", Position::dummy()),
                        hir::None::new(Position::dummy()),
                        Position::dummy(),
                    ),
                    true,
                )]
            )
        );
    }

    #[test]
    fn do_not_rename_private_type_definition() {
        let module = hir::Module::new(
            vec![hir::TypeDefinition::without_source(
                "Foo",
                vec![types::RecordElement::new(
                    "foo",
                    types::Reference::new("Bar'Bar", Position::dummy()),
                )],
                false,
                false,
                false,
            )],
            vec![],
            vec![],
            vec![hir::Definition::without_source(
                "Foo",
                hir::Lambda::new(
                    vec![],
                    types::Reference::new("Bar'Bar", Position::dummy()),
                    hir::None::new(Position::dummy()),
                    Position::dummy(),
                ),
                true,
            )],
        );

        assert_eq!(
            rename(
                &module,
                &vec![(
                    ast::InternalModulePath::new(vec!["Bar".into()]).into(),
                    interface::Module::new(
                        vec![interface::TypeDefinition::new(
                            "RealBar",
                            "Bar",
                            vec![],
                            false,
                            false,
                            Position::dummy()
                        )],
                        vec![],
                        vec![]
                    )
                )]
                .into_iter()
                .collect(),
                &[]
            ),
            module
        );
    }

    #[test]
    fn do_not_rename_private_type_alias() {
        let module = hir::Module::new(
            vec![hir::TypeDefinition::without_source(
                "Foo",
                vec![types::RecordElement::new(
                    "foo",
                    types::Reference::new("Bar'Bar", Position::dummy()),
                )],
                false,
                false,
                false,
            )],
            vec![],
            vec![],
            vec![hir::Definition::without_source(
                "Foo",
                hir::Lambda::new(
                    vec![],
                    types::Reference::new("Bar'Bar", Position::dummy()),
                    hir::None::new(Position::dummy()),
                    Position::dummy(),
                ),
                true,
            )],
        );

        assert_eq!(
            rename(
                &module,
                &vec![(
                    ast::InternalModulePath::new(vec!["Bar".into()]).into(),
                    interface::Module::new(
                        vec![],
                        vec![interface::TypeAlias::new(
                            "RealBar",
                            "Bar",
                            types::None::new(Position::dummy()),
                            false,
                        )],
                        vec![]
                    )
                )]
                .into_iter()
                .collect(),
                &[],
            ),
            module
        );
    }
}
