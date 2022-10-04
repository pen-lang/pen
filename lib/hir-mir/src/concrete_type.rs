use crate::{context::Context, type_, CompileError};
use hir::{analysis::type_canonicalizer, types::Type};

pub fn compile(
    context: &Context,
    expression: mir::ir::Expression,
    type_: &Type,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(
        match &type_canonicalizer::canonicalize(type_, context.types())? {
            Type::Boolean(_)
            | Type::Error(_)
            | Type::None(_)
            | Type::Number(_)
            | Type::Record(_)
            | Type::String(_) => expression,
            Type::Function(function_type) => mir::ir::Record::new(
                type_::compile_concrete_function(function_type, context.types())?,
                vec![expression],
            )
            .into(),
            Type::List(list_type) => mir::ir::Record::new(
                type_::compile_concrete_list(list_type, context.types())?,
                vec![expression],
            )
            .into(),
            Type::Map(map_type) => mir::ir::Record::new(
                type_::compile_concrete_map(map_type, context.types())?,
                vec![expression],
            )
            .into(),
            Type::Any(_) | Type::Reference(_) | Type::Union(_) => unreachable!(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::{ir::*, test::ModuleFake, types};
    use position::{test::PositionFake, Position};

    #[test]
    fn compile_boolean() {
        let context = Context::new(&Module::empty(), None);

        assert_eq!(
            compile(
                &context,
                mir::ir::Variable::new("x").into(),
                &types::Boolean::new(Position::fake()).into()
            ),
            Ok(mir::ir::Variable::new("x").into())
        );
    }

    #[test]
    fn compile_function() {
        let context = Context::new(&Module::empty(), None);
        let function_type = types::Function::new(
            vec![],
            types::None::new(Position::fake()),
            PositionFake::fake(),
        );

        assert_eq!(
            compile(
                &context,
                mir::ir::Variable::new("x").into(),
                &function_type.clone().into()
            ),
            Ok(mir::ir::Record::new(
                type_::compile_concrete_function(&function_type, context.types()).unwrap(),
                vec![mir::ir::Variable::new("x").into()]
            )
            .into())
        );
    }

    #[test]
    fn compile_list() {
        let context = Context::new(&Module::empty(), None);
        let list_type = types::List::new(types::None::new(Position::fake()), PositionFake::fake());

        assert_eq!(
            compile(
                &context,
                mir::ir::Variable::new("x").into(),
                &list_type.clone().into()
            ),
            Ok(mir::ir::Record::new(
                type_::compile_concrete_list(&list_type, context.types()).unwrap(),
                vec![mir::ir::Variable::new("x").into()]
            )
            .into())
        );
    }

    #[test]
    fn compile_map() {
        let context = Context::new(&Module::empty(), None);
        let map_type = types::Map::new(
            types::None::new(Position::fake()),
            types::None::new(Position::fake()),
            PositionFake::fake(),
        );

        assert_eq!(
            compile(
                &context,
                mir::ir::Variable::new("x").into(),
                &map_type.clone().into()
            ),
            Ok(mir::ir::Record::new(
                type_::compile_concrete_map(&map_type, context.types()).unwrap(),
                vec![mir::ir::Variable::new("x").into()]
            )
            .into())
        );
    }
}
