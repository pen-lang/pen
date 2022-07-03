#[cfg(test)]
use once_cell::sync::Lazy;

#[cfg(test)]
pub static LIST_TYPE_CONFIGURATION: Lazy<ListTypeConfiguration> =
    Lazy::new(|| ListTypeConfiguration {
        empty_list_function_name: "emptyList".into(),
        concatenate_function_name: "concatenateLists".into(),
        equal_function_name: "equalLists".into(),
        prepend_function_name: "prependToLists".into(),
        deconstruct_function_name: "deconstruct".into(),
        lazy_function_name: "lazy".into(),
        first_function_name: "first".into(),
        rest_function_name: "rest".into(),
        list_type_name: "GenericList".into(),
        first_rest_type_name: "FirstRest".into(),
        size_function_name: "listSize".into(),
    });

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListTypeConfiguration {
    pub empty_list_function_name: String,
    pub concatenate_function_name: String,
    pub equal_function_name: String,
    pub prepend_function_name: String,
    pub deconstruct_function_name: String,
    pub lazy_function_name: String,
    pub first_function_name: String,
    pub rest_function_name: String,
    pub list_type_name: String,
    pub first_rest_type_name: String,
    pub size_function_name: String,
}
