use std::fmt::Display;

use serde::{Deserialize, Deserializer, Serialize, de::Visitor, ser::SerializeSeq};

use crate::ascii::{achar::AChar, error::ASCIIError};

#[derive(Clone, Debug, Hash, PartialOrd)]
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

	pub fn new_of_size(size: usize) -> Self {
		let v: &mut Vec<AChar> = &mut vec![];
		v.resize(size, AChar(0x00 as u8));
		Self {
			achars: v.clone(),
			fixed_length: false,
		}
	}

	pub fn fixed(length: usize) -> Self {
		let v: &mut Vec<AChar> = &mut vec![];
		v.resize(length, AChar(0x00 as u8));
		Self {
			achars: v.clone(),
			fixed_length: true,
		}
	}

	pub fn len(&self) -> usize {
		self.achars.len()
	}

	pub fn resize(&mut self, size: usize) -> Result<(), ASCIIError> {
		if self.fixed_length {
			Err(ASCIIError {
				message: "attepted to change the length of a fixed length AString",
				offset: None,
			})
		} else {
			self.achars.resize(size, AChar::null());
			Ok(())
		}
	}
}

impl PartialEq<AString> for AString {
	fn eq(&self, other: &AString) -> bool {
		self.achars == other.achars
	}
}

impl PartialEq<&str> for AString {
	fn eq(&self, other: &&str) -> bool {
		let s = String::from(*other);
		s.as_str() == *other
	}
}

impl Display for AString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for c in &self.achars {
			f.write_str(format!("{}", c).as_str()).unwrap();
		}
		Ok(())
	}
}

impl From<&[u8]> for AString {
	fn from(value: &[u8]) -> Self {
		Self {
			achars: value.iter().map(|v| AChar(v.clone())).collect(),
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
			let s = &mut Self::new_of_size(value.len());
			for c in value.chars() {
				s.achars.push(AChar(c as u8));
			}
			Ok(s.clone())
		}
	}
}

impl From<AString> for String {
	fn from(value: AString) -> Self {
		let s = &mut String::new();
		for c in value.achars {
			s.push(c.char());
		}
		s.clone()
	}
}

impl Serialize for AString {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut seq = serializer.serialize_seq(Some(self.achars.len()))?;
		for e in &self.achars {
			seq.serialize_element(e)?;
		}
		seq.end()
	}
}

impl<'de> Deserialize<'de> for AString {
	fn deserialize<D>(deserializer: D) -> Result<AString, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_bytes(AStringVisitor)
	}
}

pub struct AStringVisitor;

impl<'de> Visitor<'de> for AStringVisitor {
	type Value = AString;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("a string of ASCII characters")
	}

	fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(AString::from(v))
	}
}

#[cfg(test)]
mod tests {
	use crate::ascii::{achar::AChar, astring::AString};

	#[test]
	fn test_create() {
		let s1 = AString::new();
		assert_eq!(s1.len(), 0);
		let s2 = AString::new_of_size(10);
		assert_eq!(s2.len(), 10);
		let s3 = AString::fixed(10);
		assert_eq!(s3.len(), 10);
	}

	#[test]
	fn test_fixed_length() {
		let s = &mut AString::fixed(10);
		assert!(s.resize(11).is_err());
	}

	#[test]
	fn test_resize() {
		let s = &mut AString::new();
		assert_eq!(s.len(), 0);
		s.resize(10).unwrap();
		assert_eq!(s.len(), 10);
	}

	#[test]
	fn test_from() {
		let s1 = AString::from(&[0x41 as u8] as &[u8]);
		assert_eq!(s1, "A");
		let s2 = AString::from(&vec![AChar(0x41 as u8)]);
		assert_eq!(s2, "A");
		let s3 = AString::try_from(&String::from("A")).unwrap();
		assert_eq!(s3, "A");
		let s4 = String::from(s3);
		assert_eq!(s4, "A");
	}
}
