use std::fs::OpenOptions;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OpenFileOptions {
    pub append: bool,
    pub create: bool,
    pub create_new: bool,
    pub read: bool,
    pub truncate: bool,
    pub write: bool,
}

impl From<OpenFileOptions> for OpenOptions {
    fn from(options: OpenFileOptions) -> Self {
        let mut open_options = Self::new();

        // Set the create option after the create_new option because the latter is
        // prioritized.
        open_options
            .append(options.append)
            .create_new(options.create_new)
            .create(options.create)
            .read(options.read)
            .truncate(options.truncate)
            .write(options.write);

        open_options
    }
}
