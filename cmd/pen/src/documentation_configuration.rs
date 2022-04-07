use crate::application_configuration::APPLICATION_CONFIGURATION;
use app::package_documentation_generator::DocumentationConfiguration;
use once_cell::sync::Lazy;

pub static DOCUMENTATION_CONFIGURATION: Lazy<DocumentationConfiguration> =
    Lazy::new(|| DocumentationConfiguration {
        language: "pen".into(),
        private_names: [APPLICATION_CONFIGURATION
            .main_module
            .new_system_context_function_name
            .to_string()]
        .into_iter()
        .collect(),
    });
