use once_cell::sync::Lazy;

pub const MODULE_LOCAL_SPAWN_FUNCTION_NAME: &str = "_local_pen_spawn";

#[cfg(test)]
pub static CONCURRENCY_CONFIGURATION: Lazy<ConcurrencyConfiguration> =
    Lazy::new(|| ConcurrencyConfiguration {
        spawn_function_name: "spawn".into(),
    });

pub static DUMMY_CONCURRENCY_CONFIGURATION: Lazy<ConcurrencyConfiguration> =
    Lazy::new(|| ConcurrencyConfiguration {
        spawn_function_name: "<dummy>".into(),
    });

#[derive(Clone, Debug)]
pub struct ConcurrencyConfiguration {
    pub spawn_function_name: String,
}
