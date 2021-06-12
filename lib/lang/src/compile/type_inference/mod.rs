mod error;

use super::type_context::TypeContext;
use crate::hir::*;
pub use error::TypeInferenceError;

pub fn infer_types(module: &Module) -> Result<Module, TypeInferenceError> {
    let context = TypeContext::new(module);

    Ok(module.clone())
}

fn infer_expression(expression: &Expression) -> Result<Expression, TypeInferenceError> {
    Ok(match expression {
        Expression::Lambda(lambda) => Lambda::new(
            lambda.arguments().to_vec(),
            infer_block(lambda.body())?,
            lambda.type_().clone(),
            lambda.position().clone(),
        )
        .into(),
        _ => todo!(),
    })
}

fn infer_block(block: &Block) -> Result<Block, TypeInferenceError> {
    Ok(match block.statements() {
        [] => todo!(),
        [statement, ..] => todo!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_empty_module() -> Result<(), TypeInferenceError> {
        infer_types(&Module::new(vec![], vec![], vec![], vec![]))?;

        Ok(())
    }
}
