#[cfg(test)]
use once_cell::sync::Lazy;

pub const MODULE_LOCAL_SPAWN_FUNCTION_NAME: &str = "__module_local_spawn";

#[cfg(test)]
pub static CONCURRENCY_CONFIGURATION: Lazy<ConcurrencyConfiguration> =
    Lazy::new(|| ConcurrencyConfiguration {
        spawn_function_name: "spawn".into(),
    });

#[derive(Clone, Debug)]
pub struct ConcurrencyConfiguration {
    pub spawn_function_name: String,
}
