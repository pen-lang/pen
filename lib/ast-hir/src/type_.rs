use hir::types::{self, Type};

pub fn compile(type_: &ast::types::Type) -> Type {
    match type_ {
        ast::types::Type::Function(function) => types::Function::new(
            function.arguments().iter().map(compile).collect(),
            compile(function.result()),
            function.position().clone(),
        )
        .into(),
        ast::types::Type::List(list) => {
            types::List::new(compile(list.element()), list.position().clone()).into()
        }
        ast::types::Type::Map(map) => types::Map::new(
            compile(map.key()),
            compile(map.value()),
            map.position().clone(),
        )
        .into(),
        ast::types::Type::Record(record) => {
            types::Record::new(record.name(), record.name(), record.position().clone()).into()
        }
        ast::types::Type::Reference(reference) => {
            types::Reference::new(reference.name(), reference.position().clone()).into()
        }
        ast::types::Type::Union(union) => types::Union::new(
            compile(union.lhs()),
            compile(union.rhs()),
            union.position().clone(),
        )
        .into(),
    }
}
