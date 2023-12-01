use crate::snow_flake_id::SnowflakeIdError::{Increment, MachineId, Timestamp};
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq)]
#[cfg_attr(test, derive(strum_macros::EnumIter))]
pub enum SnowflakeIdError {
    Timestamp,
    MachineId,
    Increment,
}

impl SnowflakeIdError {
    fn format(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Timestamp => "SnowflakeIdError::Timestamp",
            MachineId => "SnowflakeIdError::MachineId",
            Increment => "SnowflakeIdError::Increment",
        };

        write!(f, "{}", str)
    }
}

impl Debug for SnowflakeIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl Display for SnowflakeIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl std::error::Error for SnowflakeIdError {}

const MAX_TIMESTAMP: u64 = 0x03_ff_ff_ff_ff_ff;
const MAX_MACHINE_ID: u16 = 0x03_ff;
const MAX_INCLEMENT_ID: u16 = 0x0f_ff;

#[derive(PartialEq, Eq, Debug)]
pub struct SnowflakeId(u64);

impl From<u64> for SnowflakeId {
    fn from(value: u64) -> Self {
        SnowflakeId(value)
    }
}

impl From<i64> for SnowflakeId {
    fn from(value: i64) -> Self {
        SnowflakeId(value as u64)
    }
}

impl Clone for SnowflakeId {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for SnowflakeId {}

impl SnowflakeId {
    pub fn new(timestamp: u64, machine_id: u16, inclement: u16) -> Result<Self, SnowflakeIdError> {
        if timestamp > MAX_TIMESTAMP {
            Err(Timestamp)
        } else if machine_id > MAX_MACHINE_ID {
            return Err(MachineId);
        } else if inclement > MAX_INCLEMENT_ID {
            return Err(Increment);
        } else {
            let mut tmp = timestamp << 22;
            tmp |= (machine_id as u64) << 12;
            tmp |= inclement as u64;

            return Ok(SnowflakeId::from(tmp));
        }
    }

    pub fn timestamp<TzIn: TimeZone, TzOut: TimeZone>(
        &self,
        the_epoch: DateTime<TzIn>,
        time_zone: &TzOut,
    ) -> DateTime<TzOut> {
        let pivot = the_epoch.with_timezone(&Utc);

        let dur = Duration::milliseconds(self.raw_timestamp() as i64);
        (pivot + dur).with_timezone(&time_zone)
    }

    pub fn machine_id(&self) -> u16 {
        ((self.0 & 0x3F_F0_00_u64) >> 12) as u16
    }

    pub fn inclement(&self) -> u16 {
        (self.0 & 0x0F_FF_u64) as u16
    }

    pub fn raw_timestamp(&self) -> u64 {
        self.0 >> 22
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
}

impl Hash for SnowflakeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use crate::snow_flake_id::SnowflakeIdError::Timestamp;
    use crate::snow_flake_id::{
        SnowflakeId, SnowflakeIdError, MAX_INCLEMENT_ID, MAX_MACHINE_ID, MAX_TIMESTAMP,
    };
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use chrono::{DateTime, Duration, TimeZone, Utc};
    use std::sync::LazyLock;
    use strum::IntoEnumIterator;

    const SAMPLE_SCR: u64 = 175_928_847_299_678_215;
    const EXPECTED_MACHINE_ID: u16 = 169;
    const EXPECTED_INCLEMENT: u16 = 7;
    const EXPECTED_RAW_TIMESTAMP: u64 = 41_944_705_796;
    static SNOWFLAKE_EXPECTED_TIMESTAMP: LazyLock<DateTime<Utc>> = LazyLock::new(|| {
        Utc::with_ymd_and_hms(&Utc, 2016, 4, 30, 11, 18, 25)
            .unwrap()
            .checked_add_signed(Duration::milliseconds(796))
            .unwrap()
    });

    #[test]
    fn debug_display_test() {}

    static THE_EPOCH: LazyLock<DateTime<Utc>> =
        LazyLock::new(|| Utc::with_ymd_and_hms(&Utc, 2015, 1, 1, 0, 0, 0).unwrap());

    fn fixture() -> SnowflakeId {
        SnowflakeId(SAMPLE_SCR)
    }

    #[test]
    fn snowflake_id_error_debug_test() {
        let expected = [
            "SnowflakeIdError::Timestamp",
            "SnowflakeIdError::MachineId",
            "SnowflakeIdError::Increment",
        ];

        for elem in SnowflakeIdError::iter().zip(expected) {
            assert_eq!(format!("{}", elem.0), elem.1);
            assert_eq!(format!("{:?}", elem.0), elem.1);
        }
    }

    #[test]
    fn new_test() {
        let fixture = SnowflakeId::new(
            EXPECTED_RAW_TIMESTAMP,
            EXPECTED_MACHINE_ID,
            EXPECTED_INCLEMENT,
        )
        .unwrap();
        assert_eq!(fixture.0, SAMPLE_SCR);
    }

    #[test]
    fn limit_new_test() {
        let fixture = SnowflakeId::new(MAX_TIMESTAMP, MAX_MACHINE_ID, MAX_INCLEMENT_ID).unwrap();
        assert_eq!(fixture.as_u64(), u64::MAX)
    }

    #[test]
    fn our_of_range_new_test() {
        fn assert(actual: Result<SnowflakeId, SnowflakeIdError>, expected: SnowflakeIdError) {
            match actual {
                Ok(_) => unreachable!(),
                Err(e) => assert_eq!(e, expected),
            }
        }

        let fixture = SnowflakeId::new(MAX_TIMESTAMP + 1, MAX_MACHINE_ID, MAX_INCLEMENT_ID);
        assert(fixture, Timestamp);

        let fixture = SnowflakeId::new(MAX_TIMESTAMP, MAX_MACHINE_ID + 1, MAX_INCLEMENT_ID);
        assert(fixture, SnowflakeIdError::MachineId);

        let fixture = SnowflakeId::new(MAX_TIMESTAMP, MAX_MACHINE_ID, MAX_INCLEMENT_ID + 1);
        assert(fixture, SnowflakeIdError::Increment);
    }

    #[test]
    fn from_u64_test() {
        let actual = SnowflakeId::from(42u64);
        assert_eq!(actual.0, 42u64);
    }

    #[test]
    fn from_i64_test() {
        let actual = SnowflakeId::from(42i64);
        assert_eq!(actual.0, 42u64);
    }

    #[test]
    fn timestamp_test() {
        let actual = fixture();
        assert_eq!(
            actual.timestamp(*THE_EPOCH, &Utc),
            *SNOWFLAKE_EXPECTED_TIMESTAMP
        );
    }

    #[test]
    fn machine_id_test() {
        assert_eq!(fixture().machine_id(), EXPECTED_MACHINE_ID);
    }

    #[test]
    fn inclement_test_test() {
        assert_eq!(fixture().inclement(), EXPECTED_INCLEMENT);
    }

    #[test]
    fn raw_timestamp_test() {
        assert_eq!(fixture().raw_timestamp(), EXPECTED_RAW_TIMESTAMP);
    }

    #[test]
    fn as_u64_test() {
        assert_eq!(fixture().as_u64(), SAMPLE_SCR);
    }

    #[test]
    fn as_i64_test() {
        assert_eq!(fixture().as_i64(), SAMPLE_SCR as i64);
    }

    #[test]
    fn clone_test() {
        let fixture = fixture();
        let cloned = fixture.clone();

        assert_eq!(fixture, cloned)
    }

    #[test]
    fn copy_test() {
        let fixture = fixture();
        let copied = fixture;

        assert_eq!(fixture, copied)
    }

    #[test]
    fn hash_test() {
        let a = SnowflakeId::from(666324u64);
        let b = SnowflakeId::from(666324u64);

        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();

        a.hash(&mut ha);
        b.hash(&mut hb);

        assert_eq!(ha.finish(), hb.finish());

        let mut hb = DefaultHasher::new();
        SnowflakeId::from(52u64).hash(&mut hb);
        assert_ne!(ha.finish(), hb.finish());
    }
}
