const ERROR_TYPE_NAME: &str = "hir:error";

pub fn compile_type_definition() -> mir::ir::TypeDefinition {
    mir::ir::TypeDefinition::new(
        ERROR_TYPE_NAME,
        mir::types::RecordBody::new(vec![mir::types::Type::Variant]),
    )
}

pub fn compile_error(source: mir::ir::Expression) -> mir::ir::Expression {
    const VALUE_NAME: &str = "$x";
    let error_type = compile_type();

    mir::ir::Let::new(
        VALUE_NAME,
        mir::types::Type::Variant,
        source,
        mir::ir::Case::new(
            mir::ir::Variable::new(VALUE_NAME),
            vec![mir::ir::Alternative::new(
                vec![error_type.clone().into()],
                VALUE_NAME,
                mir::ir::Variable::new(VALUE_NAME),
            )],
            Some(mir::ir::DefaultAlternative::new(
                VALUE_NAME,
                mir::ir::Record::new(error_type, vec![mir::ir::Variable::new(VALUE_NAME).into()]),
            )),
        ),
    )
    .into()
}

pub fn compile_source(error: mir::ir::Expression) -> mir::ir::Expression {
    mir::ir::RecordField::new(compile_type(), 0, error).into()
}

pub fn compile_type() -> mir::types::Record {
    mir::types::Record::new(ERROR_TYPE_NAME)
}
