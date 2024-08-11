use crate::application_configuration::APPLICATION_CONFIGURATION;
use app::package_documentation_generator::DocumentationConfiguration;
use std::sync::LazyLock;

pub static DOCUMENTATION_CONFIGURATION: LazyLock<DocumentationConfiguration> =
    LazyLock::new(|| DocumentationConfiguration {
        language: "pen".into(),
        private_names: [APPLICATION_CONFIGURATION
            .main_module
            .new_system_context_function_name
            .to_string()]
        .into_iter()
        .collect(),
    });
