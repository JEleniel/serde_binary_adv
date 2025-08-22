//! Utilities for the `AChar` ASCII character type.
//!
//! The `AChar` type represents a single ASCII character.
//!
use std::fmt::Display;

// ASCII ranges
pub const MIN: u8 = 0x00;
pub const MAX: u8 = 0xFF;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AChar {
	value: u8,
}

impl AChar {
	pub fn as_utf8(&self) -> [u8; 4] {
		if self.value <= 0x7F {
			[self.value, 0x00, 0x00, 0x00]
		} else {
			[
				self.value >> 6 & 0x1F | 0xC0,
				self.value & 0x3F | 0x80,
				0x00,
				0x00,
			]
		}
	}

	pub fn len_utf8(&self) -> usize {
		match self.value {
			0x00..0x80 => 1,
			_ => 2,
		}
	}

	pub fn eq_ignore_case(&self, other: &AChar) -> bool {
		self.lowercase() == other.lowercase()
	}

	pub fn uppercase(&self) -> AChar {
		if self.is_lowercase() {
			AChar::from(&(self.value - 0x20))
		} else {
			AChar::from(&self.value)
		}
	}

	pub fn lowercase(&self) -> AChar {
		if self.is_uppercase() {
			AChar::from(&(self.value + 0x20))
		} else {
			AChar::from(&self.value)
		}
	}

	pub fn is_alphabetic(&self) -> bool {
		self.is_uppercase() || self.is_lowercase()
	}

	pub fn is_uppercase(&self) -> bool {
		match self.value {
			0x41..=0x5A => true,
			_ => false,
		}
	}

	pub fn is_lowercase(&self) -> bool {
		match self.value {
			0x61..=0x7A => true,
			_ => false,
		}
	}

	pub fn is_numeric(&self) -> bool {
		match self.value {
			0x30..=0x39 => true,
			_ => false,
		}
	}

	pub fn is_punctuation(&self) -> bool {
		match self.value {
			0x21..=0x29 | 0x3A..=0x40 | 0x5B..=0x60 | 0x7B..=0x7E => true,
			_ => false,
		}
	}

	pub fn is_control(&self) -> bool {
		match self.value {
			0x00..=0x1F => true,
			_ => false,
		}
	}

	pub fn is_whitespace(&self) -> bool {
		match self.value {
			0x09 | 0x0A | 0x0C | 0x0D | 0x20 => true,
			_ => false,
		}
	}

	pub fn is_null(&self) -> bool {
		self.value == 0x00
	}
}

impl Default for AChar {
	fn default() -> Self {
		Self { value: 0x00 }
	}
}

impl Display for AChar {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if !self.is_control() {
			write!(f, "{}", char::from(self.value))
		} else {
			Ok(())
		}
	}
}

impl From<&u8> for AChar {
	fn from(value: &u8) -> Self {
		Self {
			value: value.clone(),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::ascii::achar::{self, AChar};

	#[test]
	fn test_utf8() {
		// Test a specific, known conversion
		let nbsp = AChar::from(&(0xA0 as u8));
		assert!(nbsp.as_utf8() == [0xC2, 0xA0, 0x00, 0x00]);
		assert!(nbsp.len_utf8() == 2);

		// Test that all valid ASCII characters convert correctly
		for c in achar::MIN..=achar::MAX {
			let ch: AChar = AChar::from(&c);
			assert!(if c <= 0x7F {
				ch.as_utf8() == [c, 0x00, 0x00, 0x00]
			} else {
				ch.as_utf8() == [c >> 6 & 0x1F | 0xC0, c & 0x3F | 0x80, 0x00, 0x00]
			});
		}
	}

	#[test]
	fn test_comparisons() {
		for c in achar::MIN..=achar::MAX {
			let cha = AChar::from(&c);
			let chb = AChar::from(&c);

			assert_eq!(cha, chb);
			match c {
				0x41..=0x5A => {
					assert!(cha.is_alphabetic());
					assert!(cha.is_uppercase());
					let chc: AChar = cha.lowercase();
					assert!(cha.eq_ignore_case(&chc));
				}
				0x61..=0x7A => {
					assert!(cha.is_alphabetic());
					assert!(cha.is_lowercase());
					let chc: AChar = cha.uppercase();
					assert!(cha.eq_ignore_case(&chc));
				}
				0x30..=0x39 => {
					assert!(cha.is_numeric());
				}
				0x21..=0x29 | 0x3A..=0x40 | 0x5B..=0x60 | 0x7B..=0x7E => {
					assert!(cha.is_punctuation());
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
