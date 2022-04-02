mod ir;
mod markdown;

use ast::*;
use ir::*;

pub fn generate(module: &Module, _comments: &[Comment]) -> String {
    markdown::generate(&compile_module(module))
}

fn compile_module(_module: &Module) -> Section {
    todo!()
}
