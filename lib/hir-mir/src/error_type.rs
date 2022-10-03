const ERROR_TYPE_NAME: &str = "hir:error";

pub fn compile_type_definition() -> mir::ir::TypeDefinition {
    mir::ir::TypeDefinition::new(
        ERROR_TYPE_NAME,
        mir::types::RecordBody::new(vec![mir::types::Type::Variant]),
    )
}

pub fn compile_error(source: mir::ir::Expression) -> mir::ir::Expression {
    mir::ir::Record::new(compile_type(), vec![source]).into()
}

pub fn compile_source(error: mir::ir::Expression) -> mir::ir::Expression {
    mir::ir::RecordField::new(compile_type(), 0, error).into()
}

pub fn compile_type() -> mir::types::Record {
    mir::types::Record::new(ERROR_TYPE_NAME)
}
