use tokio::fs::OpenOptions;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct OpenFileOptions(ffi::Arc<OpenFileOptionsInner>);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct OpenFileOptionsInner {
    append: bool,
    create: bool,
    create_new: bool,
    read: bool,
    truncate: bool,
    write: bool,
}

impl From<OpenFileOptions> for OpenOptions {
    fn from(options: OpenFileOptions) -> Self {
        let options = &options.0;
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
