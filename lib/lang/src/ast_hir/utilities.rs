use crate::ast;

const PREFIX_SEPARATOR: &str = ".";

pub fn get_prefix(path: &ast::ModulePath) -> &str {
    match path {
        ast::ModulePath::External(path) => &path.components().last().unwrap(),
        ast::ModulePath::Internal(path) => &path.components().last().unwrap(),
    }
}

pub fn qualify_name(prefix: &str, name: &str) -> String {
    prefix.to_owned() + PREFIX_SEPARATOR + name
}
