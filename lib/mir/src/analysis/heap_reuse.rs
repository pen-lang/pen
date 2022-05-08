use crate::{ir::*, types::Type};
use fnv::FnvHashMap;
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
    dropped_variables: &FnvHashMap<Type, usize>,
) -> (Expression, FnvHashMap<Type, usize>) {
    match expression {
        Expression::Record(record) => {
            let mut fields = vec![];
            let mut reused_variables = FnvHashMap::default();

            for field in record.fields() {
                let (expression, variables) = convert_expression(field, &dropped_variables);

                fields.push(expression);
                reused_variables.extend(variables);
            }

            let type_ = record.type_().clone().into();

            if let Some(count) = get_dropped_variable(&dropped_variables, &type_) {
                (
                    ReusedRecord::new(
                        get_reuse_id(&type_, count),
                        Record::new(record.type_().clone(), fields),
                    )
                    .into(),
                    reused_variables,
                )
            } else {
                (expression.clone(), reused_variables)
            }
        }
        Expression::DropVariables(drop) => {
            let mut dropped_variables = dropped_variables.clone();

            for type_ in drop.variables().values() {
                add_dropped_variable(&mut dropped_variables, type_);
            }

            let (expression, reused_variables) =
                convert_expression(drop.expression(), &dropped_variables);

            (
                DropVariables::new(drop.variables().clone(), expression).into(),
                reused_variables,
            )
        }
        _ => (expression.clone(), Default::default()),
    }
}

fn add_dropped_variable(dropped_variables: &mut FnvHashMap<Type, usize>, type_: &Type) {
    update_dropped_variable(dropped_variables, type_, 1);
}

fn get_dropped_variable(variables: &FnvHashMap<Type, usize>, type_: &Type) -> Option<usize> {
    if let Some(&count) = variables.get(type_) {
        if count > 0 {
            Some(count)
        } else {
            None
        }
    } else {
        None
    }
}

fn update_dropped_variable(variables: &mut FnvHashMap<Type, usize>, type_: &Type, count: isize) {
    variables.insert(
        type_.clone(),
        (variables.get(&type_).copied().unwrap_or_default() as isize + count) as usize,
    );
}

fn get_reuse_id(type_: &Type, count: usize) -> String {
    format!("{:x}-{}", hash_type(type_), count)
}

fn hash_type(type_: &Type) -> u64 {
    let mut hasher = DefaultHasher::new();

    type_.hash(&mut hasher);

    hasher.finish()
}
