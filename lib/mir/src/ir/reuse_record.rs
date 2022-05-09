use super::Record;

#[derive(Clone, Debug, PartialEq)]
pub struct ReuseRecord {
    block_id: String,
    record: Record,
}

impl ReuseRecord {
    pub fn new(block_id: impl Into<String>, record: Record) -> Self {
        Self {
            block_id: block_id.into(),
            record,
        }
    }

    pub fn block_id(&self) -> &str {
        &self.block_id
    }

    pub fn record(&self) -> &Record {
        &self.record
    }
}
