use std::collections::HashMap;

pub fn collect(module: &ast::Module) -> HashMap<ast::ModulePath, String> {
    module
        .imports()
        .iter()
        .map(|import| (import.module_path().clone(), calculate_prefix(import)))
        .collect()
}

fn calculate_prefix(import: &ast::Import) -> String {
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
    use position::{test::PositionFake, Position};

    #[test]
    fn collect_no_prefix() {
        assert_eq!(
            collect(&ast::Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            Default::default(),
        );
    }

    #[test]
    fn collect_prefix_for_internal_module_import() {
        let path = ast::InternalModulePath::new(vec!["Foo".into()]);

        assert_eq!(
            collect(&ast::Module::new(
                vec![ast::Import::new(path.clone(), None, vec![])],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            vec![(path.into(), "Foo".into())].into_iter().collect(),
        );
    }

    #[test]
    fn collect_prefix_for_external_module_import() {
        let path = ast::ExternalModulePath::new("Foo", vec!["Bar".into()]);

        assert_eq!(
            collect(&ast::Module::new(
                vec![ast::Import::new(path.clone(), None, vec![])],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            vec![(path.into(), "Bar".into())].into_iter().collect(),
        );
    }

    #[test]
    fn collect_prefix_for_import_with_custom_prefix() {
        let path = ast::InternalModulePath::new(vec!["Foo".into()]);

        assert_eq!(
            collect(&ast::Module::new(
                vec![ast::Import::new(path.clone(), Some("Bar".into()), vec![])],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            vec![(path.into(), "Bar".into())].into_iter().collect(),
        );
    }
}
