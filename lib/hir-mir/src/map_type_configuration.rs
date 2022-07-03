#[cfg(test)]
use once_cell::sync::Lazy;

#[cfg(test)]
pub static MAP_TYPE_CONFIGURATION: Lazy<MapTypeConfiguration> =
    Lazy::new(|| MapTypeConfiguration {
        context_function_name: "newMapContext".into(),
        context_type_name: "mapContext".into(),
        delete_function_name: "deleteMap".into(),
        empty_function_name: "emptyMap".into(),
        empty_type_name: "notFound".into(),
        equal_function_name: "equalMaps".into(),
        get_function_name: "getMap".into(),
        map_type_name: "GenericMap".into(),
        merge_function_name: "mergeMaps".into(),
        set_function_name: "setMap".into(),
        size_function_name: "mapSize".into(),
        hash: HASH_CONFIGURATION.clone(),
        iteration: MAP_TYPE_ITERATION_CONFIGURATION.clone(),
    });

#[cfg(test)]
pub static HASH_CONFIGURATION: Lazy<HashConfiguration> = Lazy::new(|| HashConfiguration {
    combine_function_name: "combineHashes".into(),
    number_hash_function_name: "hashNumber".into(),
    string_hash_function_name: "hashString".into(),
    list_hash_function_name: "hashList".into(),
    map_hash_function_name: "hashMap".into(),
});

#[cfg(test)]
pub static MAP_TYPE_ITERATION_CONFIGURATION: Lazy<MapTypeIterationConfiguration> =
    Lazy::new(|| MapTypeIterationConfiguration {
        iterator_type_name: "_mapIterator".into(),
        iterate_function_name: "_iterateMap".into(),
        key_function_name: "_mapIteratorKey".into(),
        value_function_name: "_mapIteratorValue".into(),
        rest_function_name: "_mapIteratorRest".into(),
    });

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapTypeConfiguration {
    pub context_function_name: String,
    pub context_type_name: String,
    pub delete_function_name: String,
    pub empty_function_name: String,
    pub empty_type_name: String,
    pub equal_function_name: String,
    pub get_function_name: String,
    pub map_type_name: String,
    pub merge_function_name: String,
    pub set_function_name: String,
    pub size_function_name: String,
    pub hash: HashConfiguration,
    pub iteration: MapTypeIterationConfiguration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HashConfiguration {
    pub combine_function_name: String,
    pub number_hash_function_name: String,
    pub string_hash_function_name: String,
    pub list_hash_function_name: String,
    pub map_hash_function_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapTypeIterationConfiguration {
    pub iterator_type_name: String,
    pub iterate_function_name: String,
    pub key_function_name: String,
    pub value_function_name: String,
    pub rest_function_name: String,
}
