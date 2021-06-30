use once_cell::sync::Lazy;

pub const BUILD_CONFIGURATION_FILENAME: &str = "pen.json";
pub const OUTPUT_DIRECTORY: &str = ".pen";
pub const BIT_CODE_FILE_EXTENSION: &str = "bc";

pub const LANGUAGE_ROOT_HOST_NAME: &str = "pen-root";
pub const LANGUAGE_ROOT_ENVIRONMENT_VARIABLE: &str = "PEN_ROOT";

pub const FILE_PATH_CONFIGURATION: app::infra::FilePathConfiguration =
    app::infra::FilePathConfiguration {
        source_file_extension: "pen",
        object_file_extension: "o",
        interface_file_extension: "json",
        build_script_file_extension: "ninja",
    };

pub static PRELUDE_PACKAGE_CONFIGURATION: Lazy<app::infra::PreludePackageConfiguration> =
    Lazy::new(|| app::infra::PreludePackageConfiguration {
        url: url::Url::parse("file://pen-root/lib/prelude").unwrap(),
    });
