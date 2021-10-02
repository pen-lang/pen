use crate::{
    common::test_interface_serializer,
    infra::{FilePath, Infrastructure},
};
use std::{collections::BTreeMap, error::Error};

pub fn compile(
    infrastructure: &Infrastructure,
    test_interface_files: &[FilePath],
    package_test_interface_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.file_system.write(
        package_test_interface_file,
        &test_interface_serializer::serialize(
            &test_interface_files
                .iter()
                .map(|file| {
                    test_interface_serializer::deserialize(
                        &infrastructure.file_system.read_to_vec(file)?,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<BTreeMap<_, _>>(),
        )?,
    )?;

    Ok(())
}
