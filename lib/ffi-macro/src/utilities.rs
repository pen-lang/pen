use std::error::Error;
use syn::{parse_str, AttributeArgs, Lit, Meta, NestedMeta, Path};

const DEFAULT_CRATE_NAME: &str = "ffi";

pub fn parse_crate_path(attributes: &AttributeArgs) -> Result<Path, Box<dyn Error>> {
    Ok(parse_str(
        &attributes
            .iter()
            .find_map(|attribute| match attribute {
                NestedMeta::Meta(Meta::NameValue(name_value)) => {
                    if name_value.path.is_ident("crate") {
                        if let Lit::Str(string) = &name_value.lit {
                            Some(string.value())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .unwrap_or_else(|| DEFAULT_CRATE_NAME.into()),
    )?)
}
