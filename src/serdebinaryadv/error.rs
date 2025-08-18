use std;
use std::fmt::{self, Display};

use serde::{de, ser};

/// an Ok(()) or Err(serde_binary_adv::Error)
pub type Result<T> = std::result::Result<T, Error>;

/// the errors that can be thrown
#[derive(Debug)]
pub enum Error {
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
}

impl ser::Error for Error {
	fn custom<T: Display>(msg: T) -> Self {
		Error::Message(msg.to_string())
	}
}

impl de::Error for Error {
	fn custom<T: Display>(msg: T) -> Self {
		Error::Message(msg.to_string())
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::Message(msg) => write!(f, "{}", msg),
			Error::Eof => write!(f, "unexpected end of input"),
			Error::Expected(typ) => write!(f, "unexpected type, expected {}", typ),
			Error::InvalidASCII => write!(f, "nvalid ASCII character"),
			Error::InvalidUnicode => write!(f, "invalid Unicode character"),
		}
	}
}

impl std::error::Error for Error {}
