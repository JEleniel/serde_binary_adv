use crate::serdebinaryadv::common::Offset;
use std::fmt::Debug;
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
	err: Box<ErrorImpl>,
}

impl Error {
	pub fn new(kind: ErrorKind, offset: Offset, message: &str) -> Self {
		Self {
			err: Box::new(ErrorImpl {
				kind,
				offset,
				message: String::from(message),
			}),
		}
	}

	pub fn kind(&self) -> &ErrorKind {
		&self.err.kind
	}

	pub fn offset(&self) -> &Offset {
		&self.err.offset
	}

	pub fn message(&self) -> &String {
		&self.err.message
	}
}

impl Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self.err)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		Display::fmt(&*self.err, f)
	}
}

impl std::error::Error for Error {}

impl serde::ser::Error for Error {
	fn custom<T: Display>(msg: T) -> Self {
		Self {
			err: Box::new(ErrorImpl {
				kind: ErrorKind::Message,
				offset: 0,
				message: msg.to_string(),
			}),
		}
	}
}

impl serde::de::Error for Error {
	fn custom<T>(msg: T) -> Self
	where
		T: Display,
	{
		Self {
			err: Box::new(ErrorImpl {
				kind: ErrorKind::Message,
				offset: 0,
				message: msg.to_string(),
			}),
		}
	}
}

#[derive(Debug)]
struct ErrorImpl {
	kind: ErrorKind,
	offset: Offset,
	message: String,
}

impl Display for ErrorImpl {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{}: at offest {} caused by {}",
			self.kind, self.offset, self.message
		)
	}
}

#[derive(Debug)]
pub enum ErrorKind {
	Message,
	Io,
	UnexpectedEOF,
	InvalidUnicodeCodePoint,
	MarkerNotFound,
}

impl Display for ErrorKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Message => write!(f, "Message"),
			Self::Io => write!(f, "Io"),
			Self::InvalidUnicodeCodePoint => write!(f, "InvalidUnicodeCodePoint"),
			Self::UnexpectedEOF => write!(f, "UnexpectedEOF"),
			Self::MarkerNotFound => write!(f, "MarkerNotFound"),
		}
	}
}
