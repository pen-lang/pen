use std::collections::BTreeMap;

pub struct MainModuleConfiguration {
    pub source_main_function_name: String,
    pub object_main_function_name: String,
    pub context_type_name: String,
    pub contexts: BTreeMap<String, ContextConfiguration>,
}

pub struct ContextConfiguration {
    pub context_type_name: String,
    pub new_context_function_name: String,
}
