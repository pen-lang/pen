use ast::*;

// TODO Merge positions.
pub fn format(module: &Module) -> String {
    [
        module
            .imports()
            .iter()
            .map(format_import)
            .collect::<Vec<_>>()
            .join("\n"),
        module
            .foreign_imports()
            .iter()
            .map(format_foreign_import)
            .collect::<Vec<_>>()
            .join("\n"),
    ]
    .into_iter()
    .filter(|string| !string.is_empty())
    .chain(
        module
            .type_definitions()
            .iter()
            .flat_map(format_type_definition),
    )
    .chain(module.definitions().iter().flat_map(format_definition))
    .collect::<Vec<String>>()
    .join("\n\n")
        + "\n"
}

fn format_import(import: &Import) -> String {
    format!(
        "import {}{}{}",
        format_module_path(import.module_path()),
        if let Some(prefix) = import.prefix() {
            format!(" as {}", prefix)
        } else {
            "".into()
        },
        if import.unqualified_names().is_empty() {
            "".into()
        } else {
            format!(" {{ {} }}", import.unqualified_names().join(", "))
        }
    )
}

fn format_module_path(path: &ModulePath) -> String {
    match path {
        ModulePath::External(path) => {
            format!(
                "{}'{}",
                path.package(),
                format_module_path_components(path.components())
            )
        }
        ModulePath::Internal(path) => {
            format!("'{}", format_module_path_components(path.components()))
        }
    }
}

fn format_module_path_components(components: &[String]) -> String {
    components.join("'")
}

fn format_foreign_import(_import: &ForeignImport) -> String {
    todo!()
}

fn format_type_definition(_definition: &TypeDefinition) -> Vec<String> {
    todo!()
}

fn format_definition(_definition: &Definition) -> Vec<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::{test::PositionFake, Position};

    #[test]
    fn format_empty_module() {
        assert_eq!(
            format(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            "\n"
        );
    }

    #[test]
    fn format_internal_module_import() {
        assert_eq!(
            format(&Module::new(
                vec![Import::new(
                    InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                    None,
                    vec![],
                )],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            "import 'Foo'Bar\n"
        );
    }

    #[test]
    fn format_external_module_import() {
        assert_eq!(
            format(&Module::new(
                vec![Import::new(
                    ExternalModulePath::new("Package", vec!["Foo".into(), "Bar".into()]),
                    None,
                    vec![]
                )],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            "import Package'Foo'Bar\n"
        );
    }

    #[test]
    fn format_prefixed_module_import() {
        assert_eq!(
            format(&Module::new(
                vec![Import::new(
                    InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                    Some("Baz".into()),
                    vec![]
                )],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            "import 'Foo'Bar as Baz\n"
        );
    }

    #[test]
    fn format_unqualified_module_import() {
        assert_eq!(
            format(&Module::new(
                vec![Import::new(
                    InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
                    None,
                    vec!["Baz".into(), "Blah".into()]
                )],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            "import 'Foo'Bar { Baz, Blah }\n"
        );
    }
}
