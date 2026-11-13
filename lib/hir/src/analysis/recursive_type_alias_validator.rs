use super::error::AnalysisError;
use crate::{ir::*, types::Type};
use fnv::{FnvHashMap, FnvHashSet};
use petgraph::{Graph, algo::toposort};

pub fn validate(module: &Module) -> Result<(), AnalysisError> {
    let mut graph = Graph::<&str, ()>::new();
    let mut indices = FnvHashMap::<&str, _>::default();

    for alias in module.type_aliases() {
        indices.insert(alias.name(), graph.add_node(alias.name()));
    }

    for alias in module.type_aliases() {
        for name in collect_references(alias.type_()) {
            if let Some(&node) = indices.get(name) {
                graph.add_edge(indices[alias.name()], node, ());
            }
        }
    }

    toposort(&graph, None)
        .map_err(|_| AnalysisError::RecursiveTypeAlias(module.position().clone()))?;

    Ok(())
}

fn collect_references(type_: &Type) -> FnvHashSet<&str> {
    match type_ {
        Type::Function(function) => function
            .arguments()
            .iter()
            .flat_map(collect_references)
            .chain(collect_references(function.result()))
            .collect(),
        Type::List(list) => collect_references(list.element()),
        Type::Map(map) => collect_references(map.key())
            .into_iter()
            .chain(collect_references(map.value()))
            .collect(),
        Type::Union(union) => collect_references(union.lhs())
            .into_iter()
            .chain(collect_references(union.rhs()))
            .collect(),
        Type::Any(_)
        | Type::Boolean(_)
        | Type::Error(_)
        | Type::Record(_)
        | Type::None(_)
        | Type::Number(_)
        | Type::String(_) => Default::default(),
        Type::Reference(reference) => [reference.name()].into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{ModuleFake, TypeAliasFake},
        types,
    };
    use position::{Position, test::PositionFake};

    #[test]
    fn fail_to_validate_direct_recursion() {
        assert_eq!(
            validate(&Module::empty().set_type_aliases(vec![TypeAlias::fake(
                "x",
                types::Reference::new("x", Position::fake()),
                false,
                false
            )])),
            Err(AnalysisError::RecursiveTypeAlias(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_recursion_through_union() {
        assert_eq!(
            validate(&Module::empty().set_type_aliases(vec![TypeAlias::fake(
                "x",
                types::Union::new(
                    types::Reference::new("x", Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake()
                ),
                false,
                false
            )])),
            Err(AnalysisError::RecursiveTypeAlias(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_recursion_through_list() {
        assert_eq!(
            validate(&Module::empty().set_type_aliases(vec![TypeAlias::fake(
                "x",
                types::List::new(
                    types::Reference::new("x", Position::fake()),
                    Position::fake()
                ),
                false,
                false
            )])),
            Err(AnalysisError::RecursiveTypeAlias(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_recursion_through_map_key() {
        assert_eq!(
            validate(&Module::empty().set_type_aliases(vec![TypeAlias::fake(
                "x",
                types::Union::new(
                    types::Reference::new("x", Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake()
                ),
                false,
                false
            )])),
            Err(AnalysisError::RecursiveTypeAlias(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_recursion_through_map_value() {
        assert_eq!(
            validate(&Module::empty().set_type_aliases(vec![TypeAlias::fake(
                "x",
                types::Union::new(
                    types::None::new(Position::fake()),
                    types::Reference::new("x", Position::fake()),
                    Position::fake()
                ),
                false,
                false
            )])),
            Err(AnalysisError::RecursiveTypeAlias(Position::fake()))
        );
    }
}
