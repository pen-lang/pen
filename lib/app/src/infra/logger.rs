use std::error::Error;

pub trait Logger {
    fn log(&self, log: &str) -> Result<(), Box<dyn Error>>;
}
