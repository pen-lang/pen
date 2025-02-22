pub fn compile(import: &ast::Import) -> String {
    import.prefix().map(String::from).unwrap_or_else(|| {
        match import.module_path() {
            ast::ModulePath::External(path) => path.components().last().unwrap(),
            ast::ModulePath::Internal(path) => path.components().last().unwrap(),
        }
        .into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::{Position, test::PositionFake};

    #[test]
    fn calculate_prefix_for_internal_module_import() {
        let path = ast::InternalModulePath::new(vec!["Foo".into()]);

        assert_eq!(
            compile(&ast::Import::new(path, None, vec![], Position::fake())),
            "Foo",
        );
    }

    #[test]
    fn calculate_prefix_for_external_module_import() {
        assert_eq!(
            compile(&ast::Import::new(
                ast::ExternalModulePath::new("Foo", vec!["Bar".into()]),
                None,
                vec![],
                Position::fake()
            )),
            "Bar",
        );
    }

    #[test]
    fn calculate_prefix_for_import_with_custom_prefix() {
        assert_eq!(
            compile(&ast::Import::new(
                ast::InternalModulePath::new(vec!["Foo".into()]),
                Some("Bar".into()),
                vec![],
                Position::fake()
            )),
            "Bar"
        );
    }
}
