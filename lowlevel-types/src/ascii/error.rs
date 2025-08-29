use std::{error::Error, fmt::Display};

/// Represents an Error that occurred encoding or decoding ASCII values
#[derive(Debug, PartialEq)]
pub struct ASCIIError {
	/// a description of the error
	pub message: String,
}

impl Display for ASCIIError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.message)
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

#[cfg(test)]
mod tests {
	use std::error::Error;

	use crate::ascii::ASCIIError;

	#[test]
	#[allow(deprecated)]
	fn test() {
		let e = ASCIIError {
			message: String::from("test"),
		};
		assert_eq!(format!("{}", e), "test");
		assert!(e.source().is_none());
		assert!(e.cause().is_none());
		assert_eq!(e.description(), "description() is deprecated; use Display");
	}
}
