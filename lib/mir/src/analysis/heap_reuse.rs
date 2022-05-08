mod heap_block_set;

use self::heap_block_set::HeapBlockSet;
use crate::{ir::*, types::Type};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub fn reuse_heap(module: &Module) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(convert_definition)
            .collect(),
    )
}

fn convert_definition(definition: &FunctionDefinition) -> FunctionDefinition {
    let (expression, _) = convert_expression(definition.body(), &Default::default());

    FunctionDefinition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        expression,
        definition.result_type().clone(),
        definition.is_thunk(),
    )
}

fn convert_expression(
    expression: &Expression,
    dropped_blocks: &HeapBlockSet,
) -> (Expression, HeapBlockSet) {
    match expression {
        Expression::Record(record) => {
            let mut fields = vec![];
            let mut reused_blocks = HeapBlockSet::new();

            for field in record.fields() {
                let (expression, blocks) = convert_expression(field, &dropped_blocks);

                fields.push(expression);
                reused_blocks.merge(&blocks);
            }

            let type_ = record.type_().clone().into();

            if let Some(count) = dropped_blocks.get(&type_) {
                (
                    ReusedRecord::new(
                        get_reuse_id(&type_, count),
                        Record::new(record.type_().clone(), fields),
                    )
                    .into(),
                    reused_blocks,
                )
            } else {
                (expression.clone(), reused_blocks)
            }
        }
        Expression::DropVariables(drop) => {
            let mut dropped_blocks = dropped_blocks.clone();

            for type_ in drop.variables().values() {
                dropped_blocks.add(type_);
            }

            let (expression, mut reused_blocks) =
                convert_expression(drop.expression(), &dropped_blocks);

            reused_blocks.difference(&dropped_blocks);

            (todo!(), reused_blocks)
        }
        _ => (expression.clone(), Default::default()),
    }
}

fn get_reuse_id(type_: &Type, count: usize) -> String {
    format!("{:x}-{}", hash_type(type_), count)
}

fn hash_type(type_: &Type) -> u64 {
    let mut hasher = DefaultHasher::new();

    type_.hash(&mut hasher);

    hasher.finish()
}
