use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq)]
pub struct ASCIIError {
	pub message: &'static str,
	pub offset: Option<usize>,
}

impl Display for ASCIIError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.offset {
			Some(o) => {
				write!(f, "{} at offset {}", self.message, o)
			}
			None => {
				write!(f, "{}", self.message)
			}
		}
	}
}

impl Error for ASCIIError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		None
	}

	fn description(&self) -> &str {
		"description() is deprecated; use Display"
	}

	fn cause(&self) -> Option<&dyn Error> {
		self.source()
	}
}
