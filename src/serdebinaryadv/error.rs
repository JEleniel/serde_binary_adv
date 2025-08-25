use std;
use std::fmt::{self, Display};

use serde::{de, ser};

/// an Ok(()) or Err(serde_binary_adv::Error)
pub type Result<T> = std::result::Result<T, BinaryError>;

/// the errors that can be thrown
#[derive(Debug)]
pub enum BinaryError {
	/// A message only error
	Message(String),

	/// unexpected end of input
	Eof,
	/// expected a different type
	Expected(String),
	/// invalid ASCII value
	InvalidASCII,
	/// invalid Unicode code point
	InvalidUnicode,
	/// missict string terminator
	MissingStringTerminator,
	/// missing oor invalid Some/None flag
	InvalidOptionFlag,
	/// inavlign length
	InvalidLength(usize, usize),
}

impl ser::Error for BinaryError {
	fn custom<T: Display>(msg: T) -> Self {
		BinaryError::Message(msg.to_string())
	}
}

impl de::Error for BinaryError {
	fn custom<T: Display>(msg: T) -> Self {
		BinaryError::Message(msg.to_string())
	}
}

impl Display for BinaryError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			BinaryError::Message(msg) => write!(f, "{}", msg),
			BinaryError::Eof => write!(f, "unexpected end of input"),
			BinaryError::Expected(typ) => write!(f, "unexpected type, expected {}", typ),
			BinaryError::InvalidASCII => write!(f, "nvalid ASCII character"),
			BinaryError::InvalidUnicode => write!(f, "invalid Unicode character"),
			BinaryError::MissingStringTerminator => write!(f, "missing string terminator"),
			BinaryError::InvalidOptionFlag => write!(f, "missing or invalid some/none flag"),
			BinaryError::InvalidLength(a, e) => {
				write!(f, "invalid length, received {}, expected {}", a, e)
			}
		}
	}
}

impl std::error::Error for BinaryError {}
