use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct MainModuleConfiguration {
    pub source_main_function_name: String,
    pub object_main_function_name: String,
    pub context_type_name: String,
    pub contexts: BTreeMap<String, ContextConfiguration>,
}

#[derive(Clone, Debug)]
pub struct ContextConfiguration {
    pub context_type_name: String,
    pub new_context_function_name: String,
}
