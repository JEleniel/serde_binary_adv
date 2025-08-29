use super::BinaryError;

pub mod flags {
	pub const NONE: u8 = 0x00;
	pub const SOME: u8 = 0xFF;
	pub const UNIT_VARIANT: u8 = 0xFE;
	pub const NONUNIT_VARIANT: u8 = 0xFD;
	pub const STRUCT_VARIANT: u8 = 0xFC;
}

/// an Ok(()) or Err(serde_binary_adv::Error)
pub type Result<T> = std::result::Result<T, super::BinaryError>;

use std::mem::size_of;

/// Encodes an integer using a hybrid continuation bit and 3-bit length prefix scheme.
/// T must be an unsigned integer type (u8, u16, u32, u64).
pub fn compress_usize(value: usize) -> Vec<u8> {
	if value <= 0b01111111 {
		return vec![value.clone() as u8];
	}

	let mut v: usize = value.clone();
	let mut res: Vec<u8> = Vec::new();
	let mut byte_counter: u8 = 0;

	res.push((0b10000000 | (v & 0b01111111)) as u8);
	v = v >> 7;
	res.push((0b00011111 & v) as u8);
	v = v >> 5;

	while v > 0 {
		res.push((v & 0xFF) as u8);
		v = v >> 8;
		byte_counter += 1;
	}
	res[1] = res[1] | (byte_counter << 5);
	res
}

/// Decodes an integer from the hybrid continuation bit and 3-bit length prefix encoding
pub fn decompress_usize(bytes: &[u8]) -> Result<usize> {
	if bytes.is_empty() {
		return Err(BinaryError::InvalidLength {
			actual: 0,
			expected: size_of::<usize>(),
		});
	}

	let first_byte = bytes[0];
	if first_byte & 0b10000000 == 0 {
		return Ok(usize::from(first_byte));
	}

	if bytes.len() < 2 {
		return Err(BinaryError::InvalidLength {
			actual: bytes.len(),
			expected: size_of::<usize>(),
		});
	}

	let second_byte = bytes[1];
	let len: usize = (usize::from(second_byte) & usize::from(0b11100000 as u8)) >> 5;

	if bytes.len() < (len + 2) {
		return Err(BinaryError::InvalidLength {
			actual: bytes.len(),
			expected: size_of::<usize>(),
		});
	}

	let mut v: usize = (usize::from(first_byte) & usize::from(0b01111111 as u8))
		| ((usize::from(second_byte) & usize::from(0b00011111 as u8)) << 7);

	if len == 0 {
		return Ok(v.clone());
	}

	for i in 0..(len as u8) {
		v = v | (usize::from(bytes[(i + 2) as usize]) << (12 + i * 8));
	}
	Ok(v.clone())
}

/// These tests validate that the expected values have not been changed to preserve compatability
#[cfg(test)]
mod tests {
	use crate::serde_binary_adv::common::{
		compress_usize, decompress_usize,
		flags::{NONE, NONUNIT_VARIANT, SOME, STRUCT_VARIANT, UNIT_VARIANT},
	};

	#[test]
	fn test_values() {
		assert_eq!(NONE, 0x00);
		assert_eq!(SOME, 0xFF);
		assert_eq!(UNIT_VARIANT, 0xFE);
		assert_eq!(NONUNIT_VARIANT, 0xFD);
		assert_eq!(STRUCT_VARIANT, 0xFC);
	}

	#[test]
	fn test_usize_compress_min() {
		test_usize(0x00);
	}

	#[test]
	fn test_compress_usize_max() {
		test_usize(usize::MAX);
	}

	#[test]
	fn test_compress_usize_var() {
		let tests: Vec<usize> = vec![
			0x7F,
			0x80,
			0xFF,
			0x100,
			0x101,
			0xFFF,
			0x1000,
			0x1010,
			0xFFFFF,
			0x100000,
			0x101010,
			0xFFFFFF,
			0x1000000,
			0x1010101,
			0xFFFFFFF,
			0x10000000,
			0x10101010,
			0xFFFFFFFF,
			0x1000000000,
			0x1010101010,
			0xFFFFFFFFFF,
			0x10000000000,
			0x10101010101,
			0xFFFFFFFFFFF,
			0x100000000000,
			0x101010101010,
			0xFFFFFFFFFFFF,
			0x1000000000000,
			0x1010101010101,
			0xFFFFFFFFFFFFF,
			0x10000000000000,
			0x10101010101010,
			0xFFFFFFFFFFFFFF,
			0x100000000000000,
			0x101010101010101,
			0xFFFFFFFFFFFFFFF,
			0x1000000000000000,
		];
		for t in tests {
			test_usize(t);
		}
	}

	#[test]
	fn test_decompress_empty() {
		assert!(decompress_usize(&[]).is_err());
	}

	#[test]
	fn test_decompress_too_small() {
		assert!(decompress_usize(&[0x80]).is_err());
		assert!(decompress_usize(&[0xFF, 0xFF]).is_err());
	}

	fn test_usize(value: usize) {
		let encoded = compress_usize(value);
		let decoded = decompress_usize(&encoded).unwrap();
		assert_eq!(value, decoded);
	}
}
