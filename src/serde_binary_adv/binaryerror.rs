use std::fmt::{self, Display};
use std::{self, string::FromUtf8Error};

use serde::{de, ser};

/// the errors that can be thrown
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryError {
	/// A message only error
	Message { message: String },

	/// unexpected end of input
	UnexpectedEndOfInput,
	/// an invalid set of bytes for the type expected
	InvalidBytes,
	/// missing or invalid flag
	MissingOrInvalidFlag { actual: u8, expected: u8 },
	/// inavlid length
	InvalidLength { actual: usize, expected: usize },
	/// invalid name
	InvalidName { actual: String, expected: String },
	/// unexpected type
	UnexpectedType,
}

impl ser::Error for BinaryError {
	fn custom<T: Display>(msg: T) -> Self {
		BinaryError::Message {
			message: msg.to_string(),
		}
	}
}

impl de::Error for BinaryError {
	fn custom<T: Display>(msg: T) -> Self {
		BinaryError::Message {
			message: msg.to_string(),
		}
	}
}

impl Display for BinaryError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			BinaryError::Message { message } => write!(f, "{}", message),
			BinaryError::UnexpectedEndOfInput => write!(f, "unexpected end of input"),
			BinaryError::InvalidBytes => write!(f, "invalid byte sequence"),
			BinaryError::MissingOrInvalidFlag { actual, expected } => write!(
				f,
				"missing or invalid type flag, actual 0x{:X}, expected 0x{:X}",
				actual, expected
			),
			BinaryError::InvalidLength { actual, expected } => {
				write!(
					f,
					"invalid length, actual {}, expected {}",
					actual, expected
				)
			}
			BinaryError::InvalidName { actual, expected } => {
				write!(f, "invalid name, actual {}, expected {}", actual, expected)
			}
			BinaryError::UnexpectedType => write!(f, "unexpected type"),
		}
	}
}

impl std::error::Error for BinaryError {}

impl From<FromUtf8Error> for BinaryError {
	fn from(e: FromUtf8Error) -> Self {
		BinaryError::Message {
			message: format!("{:?}", e),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::BinaryError;

	#[test]
	fn test_display() {
		test_display_specific(
			BinaryError::Message {
				message: String::from("this is a test"),
			},
			"this is a test",
		);
		test_display_specific(BinaryError::InvalidBytes, "invalid byte sequence");
		test_display_specific(BinaryError::UnexpectedEndOfInput, "unexpected end of input");
		test_display_specific(BinaryError::UnexpectedType, "unexpected type");
		test_display_specific(BinaryError::UnexpectedEndOfInput, "unexpected end of input");
		test_display_specific(
			BinaryError::InvalidLength {
				actual: 2,
				expected: 1,
			},
			"invalid length, actual 2, expected 1",
		);
		test_display_specific(
			BinaryError::InvalidName {
				actual: String::from("actual"),
				expected: String::from("expected"),
			},
			"invalid name, actual actual, expected expected",
		);
		test_display_specific(
			BinaryError::MissingOrInvalidFlag {
				actual: 0xff,
				expected: 0x80,
			},
			"missing or invalid type flag, actual 0xFF, expected 0x80",
		);
	}

	fn test_display_specific(error: BinaryError, expected: &str) {
		assert_eq!(format!("{}", error), expected);
	}

	#[test]
	fn test_error() {
		let e = <BinaryError as serde::ser::Error>::custom("test");
		test_display_specific(e, "test");
		let f = <BinaryError as serde::de::Error>::custom("test");
		test_display_specific(f, "test");
	}
}
