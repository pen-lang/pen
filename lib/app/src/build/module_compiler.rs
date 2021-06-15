use super::build_context::BuildContext;
use crate::infra::FilePath;

pub fn compile_module<M, MM, I>(
    context: &BuildContext<M, MM, I>,
    source_file_path: &FilePath,
    object_file_path: &FilePath,
    interface_file_path: &FilePath,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = context.file_system.read_to_string(source_file_path)?;
    let module = context.module_parser.parse(&source, source_file_path)?;

    if self.file_system.exists(&object_file_path) {
        return Ok((object_file_path, interface_file_path));
    }

    let module_path = self.file_path_resolver.resolve_module_path(
        &source_file_path.relative_to(package_configuration.directory_path()),
        package_configuration.package(),
    );

    context
        .file_system
        .write(&object_file_path, &module_object_data)?;
    context.file_system.write(
        &interface_file_path,
        serde_json::to_string(&module_interface)?.as_bytes(),
    )?;

    Ok(())
}
