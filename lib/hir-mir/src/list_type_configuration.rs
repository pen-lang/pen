#[cfg(test)]
use std::sync::LazyLock;

#[cfg(test)]
pub static LIST_TYPE_CONFIGURATION: LazyLock<ListTypeConfiguration> =
    LazyLock::new(|| ListTypeConfiguration {
        empty_function_name: "emptyList".into(),
        concatenate_function_name: "concatenateLists".into(),
        equal_function_name: "equalLists".into(),
        maybe_equal_function_name: "maybeEqualLists".into(),
        prepend_function_name: "prependToLists".into(),
        deconstruct_function_name: "deconstruct".into(),
        lazy_function_name: "lazy".into(),
        first_function_name: "first".into(),
        rest_function_name: "rest".into(),
        list_type_name: "anyList".into(),
        first_rest_type_name: "firstRest".into(),
        size_function_name: "listSize".into(),
        debug_function_name: "debugList".into(),
    });

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListTypeConfiguration {
    pub empty_function_name: String,
    pub concatenate_function_name: String,
    pub equal_function_name: String,
    pub maybe_equal_function_name: String,
    pub prepend_function_name: String,
    pub deconstruct_function_name: String,
    pub lazy_function_name: String,
    pub first_function_name: String,
    pub rest_function_name: String,
    pub list_type_name: String,
    pub first_rest_type_name: String,
    pub size_function_name: String,
    pub debug_function_name: String,
}
