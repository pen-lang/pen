use super::Record;

#[derive(Clone, Debug, PartialEq)]
pub struct ReuseRecord {
    id: String,
    record: Record,
}

impl ReuseRecord {
    pub fn new(id: impl Into<String>, record: Record) -> Self {
        Self {
            id: id.into(),
            record,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn record(&self) -> &Record {
        &self.record
    }
}
