use super::FilePath;

pub trait DependencyCompiler {
    fn compile(&self, object_file: &FilePath, interface_files: &[FilePath]) -> String;
}
