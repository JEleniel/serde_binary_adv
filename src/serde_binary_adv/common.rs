pub mod flags {
	pub const NONE: u8 = 0x00;
	pub const SOME: u8 = 0xFF;
	pub const UNIT_VARIANT: u8 = 0xFE;
	pub const STRUCT: u8 = 0xFD;
	pub const STRUCT_VARIANT: u8 = 0xFC;
}

/// an Ok(()) or Err(serde_binary_adv::Error)
pub type Result<T> = std::result::Result<T, super::BinaryError>;

/// These tests validate that the expected values have not been changed to preserve compatability
#[cfg(test)]
mod tests {
	use crate::serde_binary_adv::common::flags::{
		NONE, SOME, STRUCT, STRUCT_VARIANT, UNIT_VARIANT,
	};

	#[test]
	fn test_values() {
		assert_eq!(NONE, 0x00);
		assert_eq!(SOME, 0xFF);
		assert_eq!(UNIT_VARIANT, 0xFE);
		assert_eq!(STRUCT, 0xFD);
		assert_eq!(STRUCT_VARIANT, 0xFC);
	}
}
