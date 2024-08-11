#[cfg(test)]
use std::sync::LazyLock;

#[cfg(test)]
pub static CONFIGURATION: Lazy<Configuration> = Lazy::new(|| Configuration {
    yield_function_name: "mir_yield".into(),
});

#[derive(Clone, Debug)]
pub struct Configuration {
    pub yield_function_name: String,
}
