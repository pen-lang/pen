use std::fs::OpenOptions;

#[derive(Clone, Copy, Debug)]
pub struct OpenFileOptions {
    pub read: bool,
    pub write: bool,
}

impl From<&OpenFileOptions> for OpenOptions {
    fn from(options: &OpenFileOptions) -> Self {
        let mut open_options = OpenOptions::new();
        open_options.read(options.read).write(options.write);
        open_options
    }
}
