use crate::snow_flake_id::SnowflakeId;
use crate::snowflake_error::SnowflakeIdEGeneratorError;
use crate::snowflake_error::SnowflakeIdEGeneratorError::MachineIdOutOfRange;
use crate::timestamp::Timestamp;
use chrono::{DateTime, TimeZone, Utc};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;

const MAX_MACHINE_ID: u16 = 1023;
const MAX_INCLEMENT_NUMBER: u16 = 4095;

pub struct SnowFlakeIdGenerator<T: Timestamp> {
    timestamp: T,
    the_epoch: DateTime<Utc>,
    machine_id: u16,
    recent: AtomicU64,
}

impl<T: Timestamp> SnowFlakeIdGenerator<T> {
    pub fn new<Tz: TimeZone>(
        timestamp: T,
        the_epoch: DateTime<Tz>,
        machine_id: u16,
    ) -> Result<Self, SnowflakeIdEGeneratorError> {
        if machine_id > MAX_MACHINE_ID {
            Err(MachineIdOutOfRange)
        } else {
            Ok(SnowFlakeIdGenerator::<T> {
                timestamp,
                the_epoch: the_epoch.with_timezone(&Utc),
                machine_id,
                recent: AtomicU64::new(0),
            })
        }
    }

    pub fn the_epoch<Tz: TimeZone>(&self, time_zone: &Tz) -> DateTime<Tz> {
        self.the_epoch.with_timezone(time_zone)
    }

    pub fn machine_id(&self) -> u16 {
        self.machine_id
    }

    fn calc_timestamp(&self, scr: DateTime<Utc>) -> u64 {
        let diff = scr - self.the_epoch;
        diff.num_milliseconds() as u64
    }

    fn try_inclement(scr: u16) -> Option<u16> {
        if scr >= MAX_INCLEMENT_NUMBER {
            None
        } else {
            Some(scr + 1)
        }
    }

    pub fn generate(&self) -> Option<SnowflakeId> {
        let pivot = SnowflakeId::from(self.recent.load(Relaxed));
        let now = self.calc_timestamp(self.timestamp.timestamp());

        let inclement = if pivot.raw_timestamp() == now {
            Self::try_inclement(pivot.inclement())?
        } else {
            0
        };

        let candidate = SnowflakeId::new(now, self.machine_id, inclement).unwrap();

        match self.recent.compare_exchange_weak(
            pivot.as_u64(),
            candidate.as_u64(),
            Relaxed,
            Relaxed,
        ) {
            Ok(_) => Some(candidate),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::timestamp::Timestamp;
    use crate::snowflake_error::SnowflakeIdEGeneratorError;
    use crate::snowflake_id_generator::SnowFlakeIdGenerator;
    use crate::timestamp::DefaultTimestamp;
    use chrono::{DateTime, Duration, FixedOffset, TimeZone, Utc};
    use mockall::mock;
    use std::ops::AddAssign;
    use std::sync::LazyLock;

    const EXPECTED_RAW_TIMESTAMP: u64 = 41_944_705_796;

    static EXPECTED_TIMESTAMP: LazyLock<DateTime<Utc>> = LazyLock::new(|| {
        Utc::with_ymd_and_hms(&Utc, 2016, 4, 30, 11, 18, 25)
            .unwrap()
            .checked_add_signed(Duration::milliseconds(796))
            .unwrap()
    });

    static DISCORD_EPOCH: LazyLock<DateTime<Utc>> =
        LazyLock::new(|| Utc::with_ymd_and_hms(&Utc, 2015, 1, 1, 0, 0, 0).unwrap());

    static THE_EPOCH: LazyLock<DateTime<Utc>> =
        LazyLock::new(|| Utc::with_ymd_and_hms(&Utc, 1970, 01, 01, 0, 0, 0).unwrap());

    mock! {
    Fixture{}
    impl Timestamp for Fixture{
        fn timestamp(&self) -> DateTime<Utc>;
    }
    }

    type MockGen = SnowFlakeIdGenerator<MockFixture>;

    #[test]
    fn calc_timestamp_test() {
        let fixture = MockGen::new(MockFixture::new(), *DISCORD_EPOCH, 1).unwrap();

        let actual = fixture.calc_timestamp(*EXPECTED_TIMESTAMP);
        assert_eq!(actual, EXPECTED_RAW_TIMESTAMP);
    }

    #[test]
    fn new_test() {
        let jst = FixedOffset::east_opt(9 * 3600).unwrap();
        let target = SnowFlakeIdGenerator::<DefaultTimestamp>::new(
            DefaultTimestamp,
            (*THE_EPOCH).with_timezone(&jst),
            10,
        )
        .unwrap();

        assert_eq!(target.the_epoch(&Utc), *THE_EPOCH);
    }

    #[test]
    fn the_epoch_test() {
        let target =
            SnowFlakeIdGenerator::<DefaultTimestamp>::new(DefaultTimestamp, *THE_EPOCH, 42)
                .unwrap();

        let jst = FixedOffset::east_opt(9 * 3600).unwrap();
        let actual = target.the_epoch(&jst);

        assert_eq!(actual, (*THE_EPOCH).with_timezone(&jst));
    }

    #[test]
    fn machine_id_test() {
        let target =
            SnowFlakeIdGenerator::<DefaultTimestamp>::new(DefaultTimestamp, *THE_EPOCH, 42)
                .unwrap();
        assert_eq!(target.machine_id(), 42);
    }

    #[test]
    fn invalid_machine_id_test() {
        let target =
            SnowFlakeIdGenerator::<DefaultTimestamp>::new(DefaultTimestamp, *THE_EPOCH, 1024);

        match target {
            Ok(_) => unreachable!(),
            Err(e) => match e {
                SnowflakeIdEGeneratorError::MachineIdOutOfRange => assert!(true),
            },
        }
    }

    #[test]
    fn generate_test() {
        let mut mock = MockFixture::new();

        mock.expect_timestamp()
            .times(4097)
            .returning(|| *EXPECTED_TIMESTAMP);

        let gen = SnowFlakeIdGenerator::new(mock, *DISCORD_EPOCH, 1).unwrap();

        for i in 0..4096u16 {
            let actual = gen.generate().unwrap();
            assert_eq!(actual.inclement(), i);
            assert_eq!(actual.raw_timestamp(), EXPECTED_RAW_TIMESTAMP);
        }

        match gen.generate() {
            None => assert!(true),
            Some(_) => unreachable!(),
        }
    }

    #[test]
    fn complex_gen_test() {
        let mut mock = MockFixture::new();
        let mut time = *THE_EPOCH;

        time.add_assign(Duration::milliseconds(1));
        let tmp = time;
        mock.expect_timestamp().times(4097).returning(move || tmp);

        time.add_assign(Duration::milliseconds(1));
        let tmp = time;

        mock.expect_timestamp().times(1).returning(move || tmp);

        let fixture = SnowFlakeIdGenerator::new(mock, *THE_EPOCH, 42).unwrap();

        for i in 0..0x1000u16 {
            let actual = fixture.generate().unwrap();
            assert_eq!(1, actual.raw_timestamp());
            assert_eq!(i, actual.inclement());
            assert_eq!(42, actual.machine_id());
        }

        assert!(fixture.generate().is_none());

        let actual = fixture.generate().unwrap();

        assert_eq!(2, actual.raw_timestamp());
        assert_eq!(42, actual.machine_id());
        assert_eq!(0, actual.inclement());
    }
}
