use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub fn calculate(url: &url::Url) -> String {
    let mut hasher = DefaultHasher::new();

    url.hash(&mut hasher);

    format!("{:x}", hasher.finish())
}
