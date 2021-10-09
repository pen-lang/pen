use crate::{
    ir::*,
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
        module
            .foreign_declarations()
            .iter()
            .map(|declaration| transform_foreign_declaration(declaration, &transform))
            .collect(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| transform_definition(definition, &transform))
            .collect(),
        module.position().clone(),
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
            definition.original_name(),
            definition
                .fields()
                .iter()
                .map(|field| types::RecordField::new(field.name(), transform(field.type_())))
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
            alias.original_name(),
            transform(alias.type_()),
            alias.is_public(),
            alias.is_external(),
            alias.position().clone(),
        )
    }
}

fn transform_foreign_declaration(
    declaration: &ForeignDeclaration,
    transform: &impl Fn(&Type) -> Type,
) -> ForeignDeclaration {
    ForeignDeclaration::new(
        declaration.name(),
        declaration.foreign_name(),
        declaration.calling_convention(),
        transform(declaration.type_()),
        declaration.position().clone(),
    )
}

fn transform_definition(definition: &Definition, transform: &impl Fn(&Type) -> Type) -> Definition {
    Definition::new(
        definition.name(),
        definition.original_name(),
        transform_lambda(definition.lambda(), transform),
        definition.foreign_definition_configuration().cloned(),
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
        transform_expression(lambda.body(), transform),
        lambda.position().clone(),
    )
}

fn transform_expression(expression: &Expression, transform: &impl Fn(&Type) -> Type) -> Expression {
    match expression {
        Expression::Call(call) => Call::new(
            call.function_type().map(transform),
            transform_expression(call.function(), transform),
            call.arguments()
                .iter()
                .map(|argument| transform_expression(argument, transform))
                .collect(),
            call.position().clone(),
        )
        .into(),
        Expression::If(if_) => If::new(
            transform_expression(if_.condition(), transform),
            transform_expression(if_.then(), transform),
            transform_expression(if_.else_(), transform),
            if_.position().clone(),
        )
        .into(),
        Expression::IfList(if_) => IfList::new(
            if_.type_().map(transform),
            transform_expression(if_.argument(), transform),
            if_.first_name(),
            if_.rest_name(),
            transform_expression(if_.then(), transform),
            transform_expression(if_.else_(), transform),
            if_.position().clone(),
        )
        .into(),
        Expression::IfType(if_) => IfType::new(
            if_.name(),
            transform_expression(if_.argument(), transform),
            if_.branches()
                .iter()
                .map(|branch| {
                    IfTypeBranch::new(
                        transform(branch.type_()),
                        transform_expression(branch.expression(), transform),
                    )
                })
                .collect(),
            if_.else_().map(|branch| {
                ElseBranch::new(
                    branch.type_().map(transform),
                    transform_expression(branch.expression(), transform),
                    branch.position().clone(),
                )
            }),
            if_.position().clone(),
        )
        .into(),
        Expression::Lambda(lambda) => transform_lambda(lambda, transform).into(),
        Expression::Let(let_) => Let::new(
            let_.name().map(String::from),
            let_.type_().map(transform),
            transform_expression(let_.bound_expression(), transform),
            transform_expression(let_.expression(), transform),
            let_.position().clone(),
        )
        .into(),
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
                .fields()
                .iter()
                .map(|field| {
                    RecordField::new(
                        field.name(),
                        transform_expression(field.expression(), transform),
                        field.position().clone(),
                    )
                })
                .collect(),
            construction.position().clone(),
        )
        .into(),
        Expression::RecordDeconstruction(deconstruction) => RecordDeconstruction::new(
            deconstruction.type_().map(transform),
            transform_expression(deconstruction.record(), transform),
            deconstruction.field_name(),
            deconstruction.position().clone(),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            transform(update.type_()),
            transform_expression(update.record(), transform),
            update
                .fields()
                .iter()
                .map(|field| {
                    RecordField::new(
                        field.name(),
                        transform_expression(field.expression(), transform),
                        field.position().clone(),
                    )
                })
                .collect(),
            update.position().clone(),
        )
        .into(),
        Expression::Thunk(thunk) => Thunk::new(
            thunk.type_().map(transform),
            transform_expression(thunk.expression(), transform),
            thunk.position().clone(),
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
            operation.type_().map(transform),
            transform_expression(operation.expression(), transform),
            operation.position().clone(),
        )
        .into(),
    }
}
