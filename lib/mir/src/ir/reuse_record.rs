use super::Record;

#[derive(Clone, Debug, PartialEq)]
pub struct ReuseRecord {
    reuse_id: String,
    record: Record,
}

impl ReuseRecord {
    pub fn new(reuse_id: impl Into<String>, record: Record) -> Self {
        Self {
            reuse_id: reuse_id.into(),
            record,
        }
    }

    pub fn reuse_id(&self) -> &str {
        &self.reuse_id
    }

    pub fn record(&self) -> &Record {
        &self.record
    }
}
