use crate::{
    common::{module_test_information_serializer, package_test_information_serializer},
    infra::{FilePath, Infrastructure},
};
use std::{collections::BTreeMap, error::Error};

pub fn compile(
    infrastructure: &Infrastructure,
    module_test_interface_files: &[FilePath],
    package_test_interface_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.file_system.write(
        package_test_interface_file,
        &package_test_information_serializer::serialize(
            &module_test_interface_files
                .iter()
                .map(|file| {
                    module_test_information_serializer::deserialize(
                        &infrastructure.file_system.read_to_vec(file)?,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|information| (information.path().into(), information))
                .collect::<BTreeMap<_, _>>(),
        )?,
    )?;

    Ok(())
}
