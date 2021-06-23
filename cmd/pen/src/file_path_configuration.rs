pub const BUILD_CONFIGURATION_FILENAME: &str = "pen.json";
pub const OUTPUT_DIRECTORY: &str = ".pen";

pub const FILE_PATH_CONFIGURATION: app::infra::FilePathConfiguration =
    app::infra::FilePathConfiguration {
        source_file_extension: "pen",
        object_file_extension: "bc",
        interface_file_extension: "json",
        build_script_file_extension: "ninja",
    };
