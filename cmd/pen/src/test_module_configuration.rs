use once_cell::sync::Lazy;

pub const TEST_MODULE_CONFIGURATION: Lazy<app::TestModuleConfiguration> =
    Lazy::new(|| app::TestModuleConfiguration {
        test_function_prefix: "_pen_test_".to_string(),
    });
