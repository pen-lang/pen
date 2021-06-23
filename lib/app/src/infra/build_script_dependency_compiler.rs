use super::FilePath;

pub trait BuildScriptDependencyCompiler {
    fn compile(&self, object_file: &FilePath, interface_files: &[FilePath]) -> String;
}
