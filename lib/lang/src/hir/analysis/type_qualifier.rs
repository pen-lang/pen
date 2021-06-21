use super::type_transformer;
use crate::{
    hir::*,
    types::{self, Type},
};
use std::collections::HashMap;

pub fn qualify(module: &Module, prefix: &str) -> Module {
    let names = module
        .type_definitions()
        .iter()
        .filter(|definition| !definition.is_external())
        .map(|definition| {
            (
                definition.name().into(),
                prefix.to_owned() + definition.name(),
            )
        })
        .chain(
            module
                .type_aliases()
                .iter()
                .filter(|alias| !alias.is_external())
                .map(|alias| (alias.name().into(), prefix.to_owned() + alias.name())),
        )
        .collect::<HashMap<String, String>>();

    // TODO Rename type definitions and aliases.
    type_transformer::transform(
        &Module::new(
            module
                .type_definitions()
                .iter()
                .map(|definition| {
                    TypeDefinition::new(
                        names
                            .get(definition.name())
                            .map(String::as_str)
                            .unwrap_or_else(|| definition.name()),
                        definition.original_name(),
                        definition.elements().to_vec(),
                        definition.is_open(),
                        definition.is_public(),
                        definition.is_external(),
                        definition.position().clone(),
                    )
                })
                .collect(),
            module
                .type_aliases()
                .iter()
                .map(|alias| {
                    TypeAlias::new(
                        names
                            .get(alias.name())
                            .map(String::as_str)
                            .unwrap_or_else(|| alias.name()),
                        alias.original_name(),
                        alias.type_().clone(),
                        alias.is_public(),
                        alias.is_external(),
                    )
                })
                .collect(),
            module.declarations().to_vec(),
            module.definitions().to_vec(),
        ),
        |type_| match type_ {
            Type::Record(record) => types::Record::new(
                names
                    .get(record.name())
                    .map(|string| string.as_str())
                    .unwrap_or_else(|| record.name()),
                record.position().clone(),
            )
            .into(),
            Type::Reference(reference) => types::Reference::new(
                names
                    .get(reference.name())
                    .map(|string| string.as_str())
                    .unwrap_or_else(|| reference.name()),
                reference.position().clone(),
            )
            .into(),
            _ => type_.clone(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn qualify_type_definition() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![TypeDefinition::without_source(
                        "x",
                        vec![],
                        false,
                        false,
                        false
                    )],
                    vec![],
                    vec![],
                    vec![],
                ),
                "foo."
            ),
            Module::new(
                vec![TypeDefinition::without_source(
                    "foo.x",
                    vec![],
                    false,
                    false,
                    false
                )],
                vec![],
                vec![],
                vec![],
            )
        );
    }

    #[test]
    fn qualify_type_definition_recursively() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![TypeDefinition::without_source(
                        "x",
                        vec![types::RecordElement::new(
                            "x",
                            types::Reference::new("x", Position::dummy())
                        )],
                        false,
                        false,
                        false
                    )],
                    vec![],
                    vec![],
                    vec![],
                ),
                "foo."
            ),
            Module::new(
                vec![TypeDefinition::without_source(
                    "foo.x",
                    vec![types::RecordElement::new(
                        "x",
                        types::Reference::new("foo.x", Position::dummy())
                    )],
                    false,
                    false,
                    false
                )],
                vec![],
                vec![],
                vec![],
            )
        );
    }

    #[test]
    fn qualify_type_alias() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![],
                    vec![TypeAlias::without_source(
                        "x",
                        types::Reference::new("x", Position::dummy()),
                        false,
                        false
                    )],
                    vec![],
                    vec![],
                ),
                "foo."
            ),
            Module::new(
                vec![],
                vec![TypeAlias::without_source(
                    "foo.x",
                    types::Reference::new("foo.x", Position::dummy()),
                    false,
                    false
                )],
                vec![],
                vec![],
            )
        );
    }
}
