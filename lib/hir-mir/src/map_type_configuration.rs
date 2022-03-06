#[cfg(test)]
use once_cell::sync::Lazy;

#[cfg(test)]
pub static MAP_TYPE_CONFIGURATION: Lazy<MapTypeConfiguration> =
    Lazy::new(|| MapTypeConfiguration {
        empty_function_name: "emptyMap".into(),
        equal_function_name: "equalMaps".into(),
        map_type_name: "GenericMap".into(),
        merge_function_name: "mergeMaps".into(),
        remove_function_name: "removeMap".into(),
        set_function_name: "setMap".into(),
    });

#[derive(Clone, Debug, PartialEq)]
pub struct MapTypeConfiguration {
    pub empty_function_name: String,
    pub equal_function_name: String,
    pub map_type_name: String,
    pub merge_function_name: String,
    pub remove_function_name: String,
    pub set_function_name: String,
}
