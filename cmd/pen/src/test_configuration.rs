use std::sync::LazyLock;

pub static TEST_CONFIGURATION: Lazy<app::TestConfiguration> =
    Lazy::new(|| app::TestConfiguration {
        test_module_configuration: app::TestModuleConfiguration {
            test_function_prefix: "_pen_test_".into(),
        },
    });
