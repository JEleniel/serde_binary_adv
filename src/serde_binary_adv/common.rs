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

use std::{mem::size_of, ops::BitAnd};

const BITS_0_3: u128 = 0b00001111;
const BITS_0_4: u128 = 0b00011111;
const BITS_0_6: u128 = 0b01111111;
const BITS_0_7: u128 = 0b11111111;
const BITS_4_7: u128 = 0b11110000;
const BITS_5_7: u128 = 0b11100000;
const BIT_7: u128 = 0b1000000;

/// Encodes a number using a hybrid continuation bit and 3-bit length prefix scheme.
/// T must be an convertible to u128.
pub fn compress<T>(value: T) -> Vec<u8>
where
	T: Into<u128> + PartialOrd<u128>,
{
	let mut res: Vec<u8> = Vec::new();
	let mut v: u128 = value.into();

	// Just return the first seven bits
	if v <= BITS_0_6 {
		res.push((v & BITS_0_6) as u8);
		return res;
	}

	// Write bytes[0]
	res.push((BIT_7 | (v & BITS_0_6)) as u8);
	v = v >> 7;

	// Write bytes[1] without size bits
	res.push((v & BITS_0_4) as u8);
	v = v >> 5;

	// Write bytes[2..=8]
	let mut byte_counter: u8 = 0;
	while v > 0 && byte_counter < 7 {
		res.push((v & BITS_0_7) as u8);
		v = v >> 8;
		byte_counter += 1;
	}

	// Write byte 9
	if v > 0 {
		res.push((v & BITS_0_3) as u8);
		v = v >> 4;
		byte_counter += 1;
	}
	res[1] = res[1] | (byte_counter << 5);

	// Write remaining bytes
	byte_counter = 0;
	while v > 0 {
		res.push((v & BITS_0_7) as u8);
		v = v >> 8;
		byte_counter += 1;
	}
	if byte_counter > 0 {
		res[8] = res[8] & (byte_counter << 4);
	}

	res
}

pub fn decompress<T>(bytes: &[u8]) -> Result<T>
where
	T: TryFrom<u128> + From<u8>,
{
	if bytes.is_empty() {
		return Err(BinaryError::InvalidLength {
			actual: 0,
			expected: size_of::<usize>(),
		});
	}

	if (bytes[0] & BIT_7 as u8) == 0 {
		return Ok(T::from(T::from(bytes[0] as u8)));
	}

	let total_len: usize = 2
		+ ((bytes[1] as u128 & BITS_5_7) >> 5) as usize
		+ if ((bytes[1] as u128 & BITS_5_7) >> 5) == 7 {
			((bytes[8] as u128 & BITS_4_7) >> 4) as usize
		} else {
			0
		};

	if bytes.len() < total_len {
		return Err(BinaryError::InvalidLength {
			actual: bytes.len(),
			expected: total_len,
		});
	}

	// Byte 0, bits 0-6
	let mut v: u128 = u128::from(bytes[0] as u128 & BITS_0_6);
	let mut shl: u8 = 7;

	// Byte 1, bits 0-4
	v = v | ((bytes[1] as u128 & BITS_0_4) as u128) << shl;
	shl += 5;

	// Bytes 2-7
	if total_len > 2 {
		for i in 2..total_len.min(7) as usize {
			v = v | (bytes[i + 2] as u128) << shl;
			shl += 8;
		}
	}

	if total_len > 8 {
		// Byte 8, bits 0-3
		v = v | (bytes[8] as u128 & BITS_0_3) << shl;
		shl += 4;
	}

	// Remaining bytes
	if total_len > 9 {
		for i in 9..total_len {
			v = v | (bytes[i + 2] as u128) << shl;
			shl += 8;
		}
	}

	Ok(v.into())
}

/// Encodes an `usize` using a hybrid continuation bit and 3-bit length prefix scheme.
/// T must be an unsigned integer type (u8, u16, u32, u64).
pub fn compress_usize(value: usize) -> Vec<u8> {
	let mut res: Vec<u8> = Vec::new();

	if value <= 0b01111111 {
		res.push(value as u8);
		return res;
	}

	let mut v: usize = value.clone();
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
	use std::ops::BitAnd;

	use crate::serde_binary_adv::common::{
		compress, compress_usize, decompress, decompress_usize,
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
	fn test_compress_t_min() {
		test(0x00);
	}

	#[test]
	fn test_compress_t_max() {
		test(u64::MAX);
	}

	#[test]
	fn test_compress_t_var() {
		let tests: Vec<u128> = vec![
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
			test(t);
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

	fn test<T>(value: T)
	where
		T: Into<u128> + From<u128> + PartialOrd<u128> + Clone + std::fmt::Debug,
	{
		let encoded: Vec<u8> = compress(value.clone());
		let decoded: T = decompress(&encoded).unwrap();
		assert_eq!(value, decoded.into());
	}

	fn test_usize(value: usize) {
		let encoded = compress_usize(value);
		let decoded = decompress_usize(&encoded).unwrap();
		assert_eq!(value, decoded);
	}
}
