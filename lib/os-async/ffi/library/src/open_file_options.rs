use tokio::fs::OpenOptions;

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

impl OpenFileOptions {
    pub fn to_tokio(&self) -> OpenOptions {
        let mut options = OpenOptions::new();

        // Set the create option after the create_new option because the latter is prioritized.
        options
            .append(self.append)
            .create_new(self.create_new)
            .create(self.create)
            .read(self.read)
            .truncate(self.truncate)
            .write(self.write);

        options
    }
}
