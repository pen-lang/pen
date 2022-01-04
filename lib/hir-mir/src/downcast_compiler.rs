use hir::{
    ir::{Expression, IfType, IfTypeBranch, Variable},
    types::Type,
};

const VALUE_NAME: &str = "$value";

pub fn compile(expression: &Expression, type_: &Type) -> Expression {
    let position = expression.position();

    IfType::new(
        VALUE_NAME,
        expression.clone(),
        vec![IfTypeBranch::new(
            type_.clone(),
            Variable::new(VALUE_NAME, position.clone()),
        )],
        None,
        position.clone(),
    )
    .into()
}
