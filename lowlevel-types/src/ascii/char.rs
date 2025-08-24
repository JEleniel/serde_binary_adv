//! Utilities for the `AChar` ASCII character type.
//!
//! The `AChar` type represents a single ASCII character.
//!
use std::fmt::Display;

use serde::{Deserialize, Deserializer, Serialize, de::Visitor};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Char(pub u8);

impl Char {
	pub fn null() -> Self {
		Char(0x00)
	}

	pub fn as_utf8(&self) -> [u8; 4] {
		if self.0 <= 0x7F {
			[self.0, 0x00, 0x00, 0x00]
		} else {
			[self.0 >> 6 & 0x1F | 0xC0, self.0 & 0x3F | 0x80, 0x00, 0x00]
		}
	}

	pub fn len_utf8(&self) -> usize {
		match self.0 {
			0x00..0x80 => 1,
			_ => 2,
		}
	}

	pub fn char(&self) -> char {
		char::from(self.0)
	}

	pub fn eq_ignore_case(&self, other: &Char) -> bool {
		self.lowercase() == other.lowercase()
	}

	pub fn uppercase(&self) -> Char {
		if self.is_lowercase() {
			Char(self.0 - 0x20)
		} else {
			Char(self.0)
		}
	}

	pub fn lowercase(&self) -> Char {
		if self.is_uppercase() {
			Char(self.0 + 0x20)
		} else {
			Char(self.0)
		}
	}

	pub fn is_alphabetic(&self) -> bool {
		self.is_uppercase() || self.is_lowercase()
	}

	pub fn is_uppercase(&self) -> bool {
		match self.0 {
			0x41..=0x5A => true,
			_ => false,
		}
	}

	pub fn is_lowercase(&self) -> bool {
		match self.0 {
			0x61..=0x7A => true,
			_ => false,
		}
	}

	pub fn is_numeric(&self) -> bool {
		match self.0 {
			0x30..=0x39 => true,
			_ => false,
		}
	}

	pub fn is_punctuation(&self) -> bool {
		match self.0 {
			0x21..=0x29 | 0x3A..=0x40 | 0x5B..=0x60 | 0x7B..=0x7E => true,
			_ => false,
		}
	}

	pub fn is_control(&self) -> bool {
		match self.0 {
			0x00..=0x1F => true,
			_ => false,
		}
	}

	pub fn is_whitespace(&self) -> bool {
		match self.0 {
			0x09 | 0x0A | 0x0C | 0x0D | 0x20 => true,
			_ => false,
		}
	}

	pub fn is_null(&self) -> bool {
		self.0 == 0x00
	}
}

impl Display for Char {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_control() {
			write!(f, "\0x{:X}", self.0)
		} else {
			write!(f, "{}", char::from(self.0))
		}
	}
}

impl From<Char> for u8 {
	fn from(value: Char) -> Self {
		value.0.clone()
	}
}

#[cfg(feature = "serde")]
impl Serialize for Char {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_u8(self.0)
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Char {
	fn deserialize<D>(deserializer: D) -> Result<Char, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_u8(ACharVisitor)
	}
}

#[cfg(feature = "serde")]
pub struct ACharVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for ACharVisitor {
	type Value = Char;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("a single ASCII character")
	}

	fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Char(v))
	}
}

#[cfg(test)]
mod tests {
	use crate::ascii;

	#[test]
	fn test_utf8() {
		// Test a specific, known conversion
		let nbsp = ascii::Char(0xA0 as u8);
		assert!(nbsp.as_utf8() == [0xC2, 0xA0, 0x00, 0x00]);
		assert!(nbsp.len_utf8() == 2);

		// Test that all valid ASCII characters convert correctly
		for c in 0x00..=0xFF {
			let ch: ascii::Char = ascii::Char(c);
			assert!(if c <= 0x7F {
				ch.as_utf8() == [c, 0x00, 0x00, 0x00]
			} else {
				ch.as_utf8() == [c >> 6 & 0x1F | 0xC0, c & 0x3F | 0x80, 0x00, 0x00]
			});
		}
	}

	#[test]
	fn test_null() {
		assert_eq!(ascii::Char::null(), ascii::Char(0x00));
	}

	#[test]
	fn test_display() {
		let cha = ascii::Char(0x00);
		assert_eq!(format!("{}", cha), "\0x0");
		let chb = ascii::Char(0x41);
		assert_eq!(format!("{}", chb), "A");
	}

	#[test]
	fn test_comparisons() {
		for c in 0x00..=0xFF {
			let cha = ascii::Char(c);
			let chb = ascii::Char(c);

			assert_eq!(cha, chb);
			match c {
				0x41..=0x5A => {
					assert!(cha.is_alphabetic());
					assert!(cha.is_uppercase());
					assert!(!cha.is_lowercase());
					assert!(!cha.is_numeric());
					assert!(!cha.is_punctuation());
					assert!(!cha.is_whitespace());
					assert!(!cha.is_control());
					let chc: ascii::Char = cha.lowercase();
					assert!(cha.eq_ignore_case(&chc));
				}
				0x61..=0x7A => {
					assert!(cha.is_alphabetic());
					assert!(cha.is_lowercase());
					assert!(!cha.is_uppercase());
					let chc: ascii::Char = cha.uppercase();
					assert!(cha.eq_ignore_case(&chc));
				}
				0x30..=0x39 => {
					assert!(cha.is_numeric());
				}
				0x21..=0x29 | 0x3A..=0x40 | 0x5B..=0x60 | 0x7B..=0x7E => {
					assert!(cha.is_punctuation());
					assert_eq!(cha, cha.uppercase());
					assert_eq!(cha, cha.lowercase());
				}
				0x00..=0x1F => {
					assert!(cha.is_control());
				}
				_ => {}
			}
			// This has to be separated because Control overlaps with Whitespace and NULL
			match c {
				0x00 => {
					assert!(cha.is_null())
				}
				0x09 | 0x0A | 0x0C | 0x0D | 0x20 => {
					assert!(cha.is_whitespace());
				}
				_ => {}
			}
		}
	}
}
