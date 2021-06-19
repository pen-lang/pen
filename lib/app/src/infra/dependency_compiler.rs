use super::FilePath;

pub trait DependencyCompiler {
    fn compile(&self, object_file: &FilePath, dependency_files: &[FilePath]) -> String;
}
