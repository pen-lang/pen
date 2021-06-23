#[cfg(test)]
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[cfg(test)]
pub static STRING_TYPE_CONFIGURATION: Lazy<StringTypeConfiguration> =
    Lazy::new(|| StringTypeConfiguration {
        equal_function_name: "equalStrings".into(),
    });

#[derive(Clone, Debug)]
pub struct StringTypeConfiguration {
    pub equal_function_name: String,
}

impl StringTypeConfiguration {
    pub fn qualify(&self, names: &HashMap<String, String>) -> Self {
        Self {
            equal_function_name: self.qualify_name(&self.equal_function_name, names),
        }
    }

    fn qualify_name(&self, name: &str, names: &HashMap<String, String>) -> String {
        names.get(name).cloned().unwrap_or_else(|| name.into())
    }
}
