use crate::{
    hir::*,
    types::{self, Type},
};

pub fn transform(module: &Module, transform: impl Fn(&Type) -> Type) -> Module {
    let transform = |type_: &Type| transform_type(type_, &transform);

    Module::new(
        module
            .type_definitions()
            .iter()
            .map(|definition| transform_type_definition(definition, &transform))
            .collect(),
        module
            .type_aliases()
            .iter()
            .map(|alias| transform_type_alias(alias, &transform))
            .collect(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| transform_definition(definition, &transform))
            .collect(),
    )
}

fn transform_type(type_: &Type, transform: &impl Fn(&Type) -> Type) -> Type {
    let transform_deeply = |type_| transform_type(type_, transform);

    transform(&match type_ {
        Type::Function(function) => types::Function::new(
            function.arguments().iter().map(transform_deeply).collect(),
            transform_deeply(function.result()),
            function.position().clone(),
        )
        .into(),
        Type::List(list) => {
            types::List::new(transform_deeply(list.element()), list.position().clone()).into()
        }
        Type::Union(union) => types::Union::new(
            transform_deeply(union.lhs()),
            transform_deeply(union.rhs()),
            union.position().clone(),
        )
        .into(),
        Type::Any(_)
        | Type::Boolean(_)
        | Type::None(_)
        | Type::Number(_)
        | Type::Record(_)
        | Type::Reference(_)
        | Type::String(_) => type_.clone(),
    })
}

fn transform_type_definition(
    definition: &TypeDefinition,
    transform: &impl Fn(&Type) -> Type,
) -> TypeDefinition {
    if definition.is_external() {
        definition.clone()
    } else {
        TypeDefinition::new(
            definition.name(),
            definition
                .elements()
                .iter()
                .map(|element| {
                    types::RecordElement::new(element.name(), transform(element.type_()))
                })
                .collect(),
            definition.is_open(),
            definition.is_public(),
            definition.is_external(),
            definition.position().clone(),
        )
    }
}

fn transform_type_alias(alias: &TypeAlias, transform: &impl Fn(&Type) -> Type) -> TypeAlias {
    if alias.is_external() {
        alias.clone()
    } else {
        TypeAlias::new(
            alias.name(),
            transform(alias.type_()),
            alias.is_public(),
            alias.is_external(),
        )
    }
}

fn transform_definition(definition: &Definition, transform: &impl Fn(&Type) -> Type) -> Definition {
    Definition::new(
        definition.name(),
        transform_lambda(definition.lambda(), transform),
        definition.is_public(),
        definition.position().clone(),
    )
}

fn transform_lambda(lambda: &Lambda, transform: &impl Fn(&Type) -> Type) -> Lambda {
    Lambda::new(
        lambda
            .arguments()
            .iter()
            .map(|argument| Argument::new(argument.name(), transform(argument.type_())))
            .collect(),
        transform(lambda.result_type()),
        transform_block(lambda.body(), transform),
        lambda.position().clone(),
    )
}

fn transform_block(block: &Block, transform: &impl Fn(&Type) -> Type) -> Block {
    Block::new(
        block
            .statements()
            .iter()
            .map(|statement| transform_statement(statement, transform))
            .collect(),
        transform_expression(block.expression(), transform),
    )
}

fn transform_statement(statement: &Statement, transform: &impl Fn(&Type) -> Type) -> Statement {
    Statement::new(
        statement.name().map(String::from),
        transform_expression(statement.expression(), transform),
        statement.type_().map(transform),
        statement.position().clone(),
    )
}

fn transform_expression(expression: &Expression, transform: &impl Fn(&Type) -> Type) -> Expression {
    match expression {
        Expression::Call(call) => Call::new(
            transform_expression(call.function(), transform),
            call.arguments()
                .iter()
                .map(|argument| transform_expression(argument, transform))
                .collect(),
            call.function_type().map(transform),
            call.position().clone(),
        )
        .into(),
        Expression::If(if_) => If::new(
            transform_expression(if_.condition(), transform),
            transform_block(if_.then(), transform),
            transform_block(if_.else_(), transform),
            if_.result_type().map(transform),
            if_.position().clone(),
        )
        .into(),
        Expression::IfList(if_) => IfList::new(
            transform_expression(if_.argument(), transform),
            if_.first_name(),
            if_.rest_name(),
            transform_block(if_.then(), transform),
            transform_block(if_.else_(), transform),
            if_.result_type().map(transform),
            if_.position().clone(),
        )
        .into(),
        Expression::IfType(if_) => IfType::new(
            if_.name(),
            transform_expression(if_.argument(), transform),
            if_.argument_type().map(transform),
            if_.branches()
                .iter()
                .map(|branch| {
                    IfTypeBranch::new(
                        branch.type_().clone(),
                        transform_block(branch.block(), transform),
                    )
                })
                .collect(),
            if_.else_().map(|block| transform_block(block, transform)),
            if_.result_type().map(transform),
            if_.position().clone(),
        )
        .into(),
        Expression::Lambda(lambda) => transform_lambda(lambda, transform).into(),
        Expression::List(list) => List::new(
            transform(list.type_()),
            list.elements()
                .iter()
                .map(|element| match element {
                    ListElement::Multiple(element) => {
                        ListElement::Multiple(transform_expression(element, transform))
                    }
                    ListElement::Single(element) => {
                        ListElement::Single(transform_expression(element, transform))
                    }
                })
                .collect(),
            list.position().clone(),
        )
        .into(),
        Expression::Operation(operation) => transform_operation(operation, transform).into(),
        Expression::RecordConstruction(construction) => RecordConstruction::new(
            transform(construction.type_()),
            construction
                .elements()
                .iter()
                .map(|(key, element)| (key.clone(), transform_expression(element, transform)))
                .collect(),
            construction.position().clone(),
        )
        .into(),
        Expression::RecordElement(element) => RecordElement::new(
            element.type_().map(transform),
            transform_expression(element.record(), transform),
            element.element_name(),
            element.position().clone(),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            transform(update.type_()),
            transform_expression(update.record(), transform),
            update
                .elements()
                .iter()
                .map(|(key, element)| (key.clone(), transform_expression(element, transform)))
                .collect(),
            update.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => TypeCoercion::new(
            transform(coercion.from()),
            transform(coercion.to()),
            transform_expression(coercion.argument(), transform),
            coercion.position().clone(),
        )
        .into(),
        Expression::Boolean(_)
        | Expression::String(_)
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::Variable(_) => expression.clone(),
    }
}

fn transform_operation(operation: &Operation, transform: &impl Fn(&Type) -> Type) -> Operation {
    match operation {
        Operation::Arithmetic(operation) => ArithmeticOperation::new(
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Boolean(operation) => BooleanOperation::new(
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Equality(operation) => EqualityOperation::new(
            operation.type_().map(transform),
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Not(operation) => NotOperation::new(
            transform_expression(operation.expression(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Order(operation) => OrderOperation::new(
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Try(operation) => TryOperation::new(
            transform_expression(operation.expression(), transform),
            operation.position().clone(),
        )
        .into(),
    }
}
