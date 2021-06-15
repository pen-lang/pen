pub trait Logger {
    fn log(&self, log: &str) -> Result<(), Box<dyn std::error::Error>>;
}
