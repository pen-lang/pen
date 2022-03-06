#[cfg(test)]
use once_cell::sync::Lazy;

#[cfg(test)]
pub static MAP_TYPE_CONFIGURATION: Lazy<MapTypeConfiguration> =
    Lazy::new(|| MapTypeConfiguration {
        empty_function_name: "emptyMap".into(),
        equal_function_name: "equalMaps".into(),
        map_type_name: "GenericMap".into(),
    });

#[derive(Clone, Debug, PartialEq)]
pub struct MapTypeConfiguration {
    pub empty_function_name: String,
    pub equal_function_name: String,
    pub map_type_name: String,
}
