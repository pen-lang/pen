use std::sync::LazyLock;

pub static TEST_CONFIGURATION: LazyLock<app::TestConfiguration> =
    LazyLock::new(|| app::TestConfiguration {
        test_module_configuration: app::TestModuleConfiguration {
            test_function_prefix: "_pen_test_".into(),
        },
    });
