use super::file_path_converter::FilePathConverter;
use std::sync::Arc;

pub struct NinjaBuildScriptDependencyCompiler {
    file_path_converter: Arc<FilePathConverter>,
}

impl NinjaBuildScriptDependencyCompiler {
    pub fn new(file_path_converter: Arc<FilePathConverter>) -> Self {
        Self {
            file_path_converter,
        }
    }
}

impl app::infra::BuildScriptDependencyCompiler for NinjaBuildScriptDependencyCompiler {
    fn compile(
        &self,
        object_file: &app::infra::FilePath,
        dependency_files: &[app::infra::FilePath],
    ) -> String {
        [
            "ninja_dyndep_version = 1".into(),
            format!(
                "build {}: dyndep | {}",
                self.file_path_converter
                    .convert_to_os_path(object_file)
                    .display(),
                dependency_files
                    .iter()
                    .map(|path| format!(
                        "{}",
                        self.file_path_converter.convert_to_os_path(path).display()
                    ))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ]
        .join("\n")
            + "\n"
    }
}
