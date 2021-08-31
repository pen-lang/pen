
use crate::ast;

pub fn qualify(path: &ast::ModulePath, name: &str) -> String {
    match path {
        ast::ModulePath::External(path) => path.components().last().unwrap(),
        ast::ModulePath::Internal(path) => path.components().last().unwrap(),
    }
    .to_owned()
        + ast::IDENTIFIER_SEPARATOR
        + name
}
