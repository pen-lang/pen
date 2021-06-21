use crate::{
    ast,
    hir::{self, analysis::variable_renamer},
    interface,
};
use std::collections::HashMap;

const PREFIX_SEPARATOR: &str = ".";

pub fn rename(
    module: &hir::Module,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
) -> hir::Module {
    rename_variables(module, module_interfaces)
}

fn rename_variables(
    module: &hir::Module,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
) -> hir::Module {
    variable_renamer::rename(
        module,
        &module_interfaces
            .iter()
            .flat_map(|(path, interface)| {
                let prefix = get_prefix(path);

                interface
                    .declarations()
                    .iter()
                    .map(|declaration| {
                        (
                            prefix.clone() + PREFIX_SEPARATOR + declaration.original_name(),
                            declaration.name().into(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect(),
    )
}

fn get_prefix(path: &ast::ModulePath) -> String {
    match path {
        ast::ModulePath::External(path) => path.components().last().unwrap().clone(),
        ast::ModulePath::Internal(path) => path.components().last().unwrap().clone(),
    }
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
                &Default::default()
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
                            hir::Block::new(
                                vec![],
                                hir::Variable::new("Bar.Bar", None, Position::dummy())
                            ),
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
                .collect()
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
                        hir::Block::new(
                            vec![],
                            hir::Variable::new("RealBar", None, Position::dummy())
                        ),
                        Position::dummy(),
                    ),
                    true,
                    Position::dummy()
                )]
            )
        );
    }
}
