use std::fmt::Display;

use serde::{Deserialize, Deserializer, Serialize, de::Visitor};

use crate::ascii::{char::Char, error::ASCIIError};

#[derive(Clone, Debug, Hash, PartialOrd)]
pub struct FixedLengthString<const N: usize>(pub [Char; N]);

impl<const N: usize> FixedLengthString<N> {
	pub fn new() -> Self {
		Self([Char(0x00); N])
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn as_bytes(&self) -> [u8; N] {
		self.0.map(|c| u8::from(c)).clone()
	}
}

impl<const N: usize> Display for FixedLengthString<N> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0
			.iter()
			.for_each(|c| f.write_str(c.char().to_string().as_str()).unwrap());
		Ok(())
	}
}

impl<const N: usize> PartialEq<FixedLengthString<N>> for FixedLengthString<N> {
	fn eq(&self, other: &FixedLengthString<N>) -> bool {
		self.0 == other.0
	}
}

impl<const N: usize> PartialEq<&str> for FixedLengthString<N> {
	fn eq(&self, other: &&str) -> bool {
		let s = String::from(self.clone());
		s.as_str() == *other
	}
}

impl<const N: usize> From<Vec<u8>> for FixedLengthString<N> {
	fn from(value: Vec<u8>) -> Self {
		FixedLengthString::<N>::try_from(value.as_slice()).unwrap()
	}
}

impl<const N: usize> TryFrom<&[u8]> for FixedLengthString<N> {
	type Error = ASCIIError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		if value.len() > N {
			Err(ASCIIError {
				message: format!(
					"array &[u8] of length {} too long for AString<{}>",
					value.len(),
					N
				),
				offset: None,
			})
		} else {
			let v: &mut [Char; N] = &mut [Char(0x00); N];
			for i in 0..value.len() {
				v[i] = Char(value[i].clone());
			}
			Ok(FixedLengthString(v.clone()))
		}
	}
}

impl<const N: usize> From<[u8; N]> for FixedLengthString<N> {
	fn from(value: [u8; N]) -> Self {
		FixedLengthString(value.map(|c| Char(c)))
	}
}

impl<const N: usize> TryFrom<&String> for FixedLengthString<N> {
	type Error = ASCIIError;

	fn try_from(value: &String) -> Result<Self, Self::Error> {
		FixedLengthString::<N>::try_from(value.as_str())
	}
}

impl<const N: usize> TryFrom<&str> for FixedLengthString<N> {
	type Error = ASCIIError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		if !value.is_ascii() {
			Err(ASCIIError {
				message: format!("attempt to convert an Unicode string to an AString"),
				offset: None,
			})
		} else {
			let v: &mut Vec<u8> = &mut Vec::new();
			for c in value.chars() {
				v.push(c as u8);
			}
			Ok(FixedLengthString::from(v.clone()))
		}
	}
}

impl<const N: usize> From<FixedLengthString<N>> for String {
	fn from(value: FixedLengthString<N>) -> Self {
		let s = &mut String::new();
		for c in value.0 {
			s.push(c.char());
		}
		s.clone()
	}
}

#[cfg(feature = "serde")]
impl<const N: usize> Serialize for FixedLengthString<N> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let bytes = &self.as_bytes() as &[u8];
		serializer.serialize_bytes(bytes)
	}
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> Deserialize<'de> for FixedLengthString<N> {
	fn deserialize<D>(deserializer: D) -> Result<FixedLengthString<N>, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_bytes(AStringVisitor::<N>)
	}
}

#[cfg(feature = "serde")]
pub struct AStringVisitor<const N: usize>;

#[cfg(feature = "serde")]
impl<'de, const N: usize> Visitor<'de> for AStringVisitor<N> {
	type Value = FixedLengthString<N>;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str(format!("an array of {} ASCII bytes", N).as_str())
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		if v.len() != N {
			Err(serde::de::Error::invalid_length(v.len(), &self))
		} else {
			Ok(FixedLengthString::try_from(v).unwrap())
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::ascii::{self, char::Char, fixedlengthstring::FixedLengthString};

	#[test]
	fn test_create() {
		let s1 = FixedLengthString([Char(0x00)]);
		assert_eq!(s1.len(), 1);
		assert_eq!(s1, "\0");
		let s: FixedLengthString<10> = FixedLengthString::new();
		assert_eq!(s.len(), 10);
	}

	#[test]
	fn test_from_vec_of_u8() {
		let v: Vec<u8> = vec![0x41 as u8];
		let s1: FixedLengthString<1> = FixedLengthString::from(v);
		assert_eq!(s1, "A");
	}

	#[test]
	fn test_from_array_of_u8() {
		let s1: FixedLengthString<1> = FixedLengthString::try_from(&[0x41 as u8] as &[u8]).unwrap();
		assert_eq!(s1, "A");
		let s2: FixedLengthString<1> = FixedLengthString::from([0x41 as u8; 1]);
		assert_eq!(s2, "A");
		assert!(FixedLengthString::<1>::try_from(&[0x41 as u8; 2] as &[u8]).is_err());
	}

	#[test]
	fn test_from_string() {
		let s1: FixedLengthString<1> = FixedLengthString::try_from(&String::from("A")).unwrap();
		assert_eq!(s1, "A");
		assert!(FixedLengthString::<1>::try_from(&String::from("👿")).is_err());
	}

	#[test]
	fn test_string_try_from_astring() {
		let s4 = String::from(FixedLengthString::<1>::try_from("A").unwrap());
		assert_eq!(s4, "A");
	}

	#[test]
	fn test_display() {
		let s = FixedLengthString([ascii::Char(0x41); 1]);
		assert_eq!(format!("{}", s), "A");
	}
}
