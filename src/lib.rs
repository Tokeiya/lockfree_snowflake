#![feature(lazy_cell)]

use chrono::{DateTime, TimeZone, Utc};
use std::sync::LazyLock;

pub mod snow_flake_id;
pub mod snowflake_error;
pub mod snowflake_id_generator;
pub mod timestamp;

pub static THE_EPOCH: LazyLock<DateTime<Utc>> =
	LazyLock::new(|| Utc::with_ymd_and_hms(&Utc, 2023, 09, 01, 0, 0, 0).unwrap());
