pub const BUILD_CONFIGURATION_FILENAME: &str = "pen.json";
pub const OUTPUT_DIRECTORY: &str = ".pen";
pub const DEFAULT_TARGET_DIRECTORY: &str = "default";
pub const DEPENDENCY_FILE_EXTENSION: &str = "d";
pub const BIT_CODE_FILE_EXTENSION: &str = "bc";
pub const NINJA_DYNAMIC_DEPENDENCY_FILE_EXTENSION: &str = "ninja";

pub const LANGUAGE_ROOT_SCHEME: &str = "pen";
pub const LANGUAGE_ROOT_ENVIRONMENT_VARIABLE: &str = "PEN_ROOT";
pub const PACKAGES_DIRECTORY: &str = "packages";

pub const DEFAULT_SYSTEM_PACKAGE_NAME: &str = "Os";
pub const DEFAULT_SYSTEM_PACKAGE_URL: &str = "pen:///os";
pub const PRELUDE_PACKAGE_URL: &str = "pen:///prelude";
pub const FFI_BUILD_SCRIPT_BASENAME: &str = "pen-ffi";
pub const LINK_SCRIPT_BASENAME: &str = "pen-link";

pub const FILE_PATH_CONFIGURATION: app::infra::FilePathConfiguration =
    app::infra::FilePathConfiguration {
        source_file_extension: "pen",
        object_file_extension: "o",
        interface_file_extension: "i",
        test_information_file_extension: "test.json",
        archive_file_extension: "a",
        build_script_file_extension: "ninja",
        test_file_extension: "test.pen",
    };
