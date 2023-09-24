use chrono::{DateTime, Utc};

pub trait Timestamp {
	fn timestamp(&self) -> DateTime<Utc>;
}

pub struct DefaultTimestamp;

impl Timestamp for DefaultTimestamp {
	fn timestamp(&self) -> DateTime<Utc> {
		Utc::now()
	}
}

impl DefaultTimestamp {
	pub fn new() -> Self {
		DefaultTimestamp
	}
}
