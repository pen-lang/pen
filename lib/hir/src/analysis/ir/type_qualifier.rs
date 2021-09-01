use super::type_transformer;
use crate::{
    ir::*,
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
                        alias.position().clone(),
                    )
                })
                .collect(),
            module.foreign_declarations().to_vec(),
            module.declarations().to_vec(),
            module.definitions().to_vec(),
            module.position().clone(),
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
    use crate::{
        test,
        test::{ModuleFake, TypeAliasFake, TypeDefinitionFake},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn qualify_type_definition() {
        assert_eq!(
            qualify(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "x",
                        vec![],
                        false,
                        false,
                        false
                    )])
                    .set_definitions(vec![]),
                "foo."
            ),
            Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "foo.x",
                    vec![],
                    false,
                    false,
                    false
                )])
                .set_definitions(vec![])
        );
    }

    #[test]
    fn qualify_type_definition_recursively() {
        assert_eq!(
            qualify(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "x",
                        vec![types::RecordElement::new(
                            "x",
                            types::Reference::new("x", test::position())
                        )],
                        false,
                        false,
                        false
                    )])
                    .set_definitions(vec![]),
                "foo."
            ),
            Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "foo.x",
                    vec![types::RecordElement::new(
                        "x",
                        types::Reference::new("foo.x", test::position())
                    )],
                    false,
                    false,
                    false
                )])
                .set_definitions(vec![])
        );
    }

    #[test]
    fn qualify_type_alias() {
        assert_eq!(
            qualify(
                &Module::empty().set_type_aliases(vec![TypeAlias::fake(
                    "x",
                    types::Reference::new("x", test::position()),
                    false,
                    false
                )]),
                "foo."
            ),
            Module::empty().set_type_aliases(vec![TypeAlias::fake(
                "foo.x",
                types::Reference::new("foo.x", test::position()),
                false,
                false
            )])
        );
    }
}
