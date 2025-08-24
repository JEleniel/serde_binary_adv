use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq)]
pub struct ASCIIError {
	pub message: String,
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

#[cfg(test)]
mod tests {
	use std::error::Error;

	use crate::ascii::ASCIIError;

	#[test]
	#[allow(deprecated)]
	fn test() {
		let e = ASCIIError {
			message: String::from("test"),
			offset: None,
		};
		assert_eq!(format!("{}", e), "test");
		assert!(e.source().is_none());
		assert!(e.cause().is_none());
		assert_eq!(e.description(), "description() is deprecated; use Display");

		let f = ASCIIError {
			message: String::from("test offset"),
			offset: Some(10),
		};
		assert_eq!(format!("{}", f), "test offset at offset 10");
	}
}
