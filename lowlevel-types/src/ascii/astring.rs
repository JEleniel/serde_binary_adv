use crate::ascii::{achar::AChar, error::ASCIIError};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AString {
	achars: Vec<AChar>,
	fixed_length: bool,
}

impl AString {
	pub fn new() -> Self {
		Self {
			achars: vec![],
			fixed_length: false,
		}
	}

	pub fn new_of_size(size: &usize) -> Self {
		Self {
			achars: Vec::<AChar>::with_capacity(*size),
			fixed_length: false,
		}
	}

	pub fn fixed(length: &usize) -> Self {
		Self {
			achars: Vec::<AChar>::with_capacity(*length),
			fixed_length: true,
		}
	}

	pub fn len(&self) -> usize {
		self.achars.len()
	}

	pub fn resize(&mut self, size: &usize) -> Result<(), ASCIIError> {
		if self.fixed_length {
			Err(ASCIIError {
				message: "attepted to change the length of a fixed length AString",
				offset: None,
			})
		} else {
			self.achars.resize(*size, AChar::null());
			Ok(())
		}
	}
}

impl From<&[u8]> for AString {
	fn from(value: &[u8]) -> Self {
		Self {
			achars: value.iter().map(|v| AChar::from(v.clone())).collect(),
			fixed_length: false,
		}
	}
}

impl From<&Vec<AChar>> for AString {
	fn from(value: &Vec<AChar>) -> Self {
		Self {
			achars: value.clone(),
			fixed_length: false,
		}
	}
}

impl TryFrom<&String> for AString {
	type Error = ASCIIError;

	fn try_from(value: &String) -> Result<Self, Self::Error> {
		if !value.is_ascii() {
			Err(ASCIIError {
				message: "attempt to convert an Unicode string to an AString",
				offset: None,
			})
		} else {
			let s = &mut Self::new_of_size(&value.len());
			for c in value.chars() {
				s.achars.push(AChar::from(c as u8));
			}
			Ok(s.clone())
		}
	}
}

impl Into<String> for AString {
	fn into(self) -> String {
		String::from_utf8(self.achars.iter().map(|c| u8::from(c)).collect()).unwrap()
	}
}
