use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub enum SnowflakeIdEGeneratorError {
	MachineIdOutOfRange,
}

#[allow(unreachable_patterns)]
fn format(this: &SnowflakeIdEGeneratorError, f: &mut Formatter<'_>) -> std::fmt::Result {
	let tmp = match this {
		SnowflakeIdEGeneratorError::MachineIdOutOfRange => "MachineIdOutOfRange",
		_ => unreachable!(),
	};
	write!(f, "SnowflakeIdEGeneratorError::{}", tmp)
}

impl Debug for SnowflakeIdEGeneratorError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		format(self, f)
	}
}

impl Display for SnowflakeIdEGeneratorError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		format(self, f)
	}
}

impl Error for SnowflakeIdEGeneratorError {}

#[cfg(test)]
mod tests {
	use crate::snowflake_error::SnowflakeIdEGeneratorError;

	#[test]
	fn debug_test() {
		let target = SnowflakeIdEGeneratorError::MachineIdOutOfRange;
		assert_eq!(
			"SnowflakeIdEGeneratorError::MachineIdOutOfRange",
			format!("{:?}", target)
		)
	}

	#[test]
	fn display_test() {
		let target = SnowflakeIdEGeneratorError::MachineIdOutOfRange;
		assert_eq!(
			"SnowflakeIdEGeneratorError::MachineIdOutOfRange",
			format!("{}", target)
		)
	}
}
