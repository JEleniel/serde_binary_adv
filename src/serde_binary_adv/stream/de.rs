use crate::serde_binary_adv::common::{
	decompress_usize,
	flags::{NONE, NONUNIT_VARIANT, SOME, STRUCT_VARIANT, UNIT_VARIANT},
};

use super::super::BinaryError;
use super::super::Result;
use serde::de::{
	self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, VariantAccess, Visitor,
};
use serde::{Deserialize, de::SeqAccess};
use std::io::Read;

macro_rules! impl_deserialize_num {
	($name:ident, $ty:ty, $visit:ident) => {
		fn $name<V>(self, visitor: V) -> Result<V::Value>
		where
			V: Visitor<'de>,
		{
			let bytes: Vec<u8> = match self.take(size_of::<$ty>()) {
				Ok(v) => v,
				Err(e) => {
					return Err(e);
				}
			};

			let value: $ty = if self.big_endian {
				<$ty>::from_be_bytes(match bytes.try_into() {
					Ok(v) => v,
					Err(e) => {
						return Err(BinaryError::Message {
							message: format!("{:?}", e),
						});
					}
				})
			} else {
				<$ty>::from_le_bytes(match bytes.try_into() {
					Ok(v) => v,
					Err(e) => {
						return Err(BinaryError::Message {
							message: format!("{:?}", e),
						});
					}
				})
			};

			visitor.$visit(value)
		}
	};
}

macro_rules! impl_next_uxx {
	($name:ident, $ty:ty) => {
		fn $name(&mut self) -> Result<$ty> {
			let bytes = match self.take(size_of::<$ty>()) {
				Ok(v) => v,
				Err(e) => {
					return Err(BinaryError::Message {
						message: format!("{:?}", e),
					});
				}
			};
			Ok(if self.big_endian {
				<$ty>::from_be_bytes(match bytes.try_into() {
					Ok(v) => v,
					Err(e) => {
						return Err(BinaryError::Message {
							message: format!("{:?}", e),
						});
					}
				})
			} else {
				<$ty>::from_le_bytes(match bytes.try_into() {
					Ok(v) => v,
					Err(e) => {
						return Err(BinaryError::Message {
							message: format!("{:?}", e),
						});
					}
				})
			})
		}
	};
}

/// Deserializes binary data into Rust types
pub struct Deserializer<'de> {
	input: &'de mut dyn Read,
	big_endian: bool,
}

impl<'de> Deserializer<'de> {
	/// Deserializes a vector of bytes (`Vec<u8>`) into Rust structures.
	pub fn read_bytes<T>(input: &'de mut dyn Read, big_endian: bool) -> Result<T>
	where
		T: Deserialize<'de>,
	{
		let mut deserializer = Deserializer::new(input, big_endian);

		let t = T::deserialize(&mut deserializer)?;
		Ok(t)
	}

	/// Creates a binary deserializer
	pub fn new(input: &'de mut dyn Read, big_endian: bool) -> Deserializer<'de> {
		Deserializer { input, big_endian }
	}

	fn next(&mut self) -> Result<u8> {
		let buf: &mut [u8] = &mut [0x00 as u8];
		match self.input.read(buf) {
			Ok(v) => {
				if v < 1 {
					Err(BinaryError::UnexpectedEndOfInput)
				} else {
					Ok(buf[0])
				}
			}
			Err(e) => Err(BinaryError::Message {
				message: e.to_string(),
			}),
		}
	}

	fn take(&mut self, len: usize) -> Result<Vec<u8>> {
		let buf: &mut Vec<u8> = &mut vec![0x00 as u8; len];
		match self.input.read(buf) {
			Ok(v) => {
				if v < len {
					Err(BinaryError::UnexpectedEndOfInput)
				} else {
					Ok(buf.to_vec())
				}
			}
			Err(e) => Err(BinaryError::Message {
				message: e.to_string(),
			}),
		}
	}

	impl_next_uxx!(next_u32, u32);

	fn next_usize(&mut self) -> Result<usize> {
		let mut bytes: Vec<u8> = vec![self.next()?];
		if (bytes[0] & 0b10000000) != 0 {
			bytes.push(self.next()?);
			let extra_bytes = (bytes[1] & 0b11100000) >> 5;
			if extra_bytes > 0 {
				for _ in 0..extra_bytes {
					bytes.push(self.next()?);
				}
			}
		}
		Ok(decompress_usize(&bytes)?)
	}

	fn take_string(&mut self) -> Result<String> {
		let size = self.next_usize()?;
		match String::from_utf8(self.take(size)?) {
			Ok(v) => Ok(v),
			Err(e) => Err(BinaryError::Message {
				message: format!("{:?}", e),
			}),
		}
	}
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
	type Error = BinaryError;

	impl_deserialize_num!(deserialize_u16, u16, visit_u16);
	impl_deserialize_num!(deserialize_u32, u32, visit_u32);
	impl_deserialize_num!(deserialize_u64, u64, visit_u64);
	impl_deserialize_num!(deserialize_u128, u128, visit_u128);

	impl_deserialize_num!(deserialize_i16, i16, visit_i16);
	impl_deserialize_num!(deserialize_i32, i32, visit_i32);
	impl_deserialize_num!(deserialize_i64, i64, visit_i64);
	impl_deserialize_num!(deserialize_i128, i128, visit_i128);

	impl_deserialize_num!(deserialize_f32, f32, visit_f32);
	impl_deserialize_num!(deserialize_f64, f64, visit_f64);

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_bool(self.next()? != 0x00)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_i8(self.next()? as i8)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_u8(self.next()?)
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let mut bytes: Vec<u8> = Vec::new();
		bytes.push(self.next()?);
		match bytes[0] {
			0x00..=0x7F => {
				//Nothing to do
			}
			0xC0..=0xDF => {
				bytes.append(&mut self.take(1)?);
			}
			0xE0..=0xEF => {
				bytes.append(&mut self.take(3)?);
			}
			0xF0..=0xFF => {
				bytes.append(&mut self.take(4)?);
			}
			_ => return Err(BinaryError::InvalidBytes),
		}
		let s = match String::from_utf8(bytes) {
			Ok(v) => v,
			Err(e) => {
				return Err(BinaryError::from(e));
			}
		};
		let ch = match s.chars().next() {
			Some(c) => c,
			None => {
				// Because of the prior checks, this should never return None
				return Err(BinaryError::Message {
					message: String::from("failed to decode character"),
				});
			}
		};
		visitor.visit_char(ch)
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let s = self.take_string()?;
		visitor.visit_str(&s.as_str())
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}

	fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let flag: u8 = self.next()?;
		if flag == NONE {
			visitor.visit_none()
		} else if flag == SOME {
			visitor.visit_some(self)
		} else {
			Err(BinaryError::MissingOrInvalidFlag {
				actual: flag,
				expected: SOME,
			})
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_unit()
	}

	// Unit struct means a named value containing no data.
	fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_unit()
	}

	fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let len: usize = self.next_usize()?;
		visitor.visit_seq(BinarySeries::new(&mut *self, len))
	}

	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		_len: usize,
		visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let len: usize = self.next_usize()?;
		visitor.visit_map(BinarySeries::new(self, len))
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let len = self.next_usize()?;
		visitor.visit_seq(BinarySeries::new(&mut *self, len))
	}

	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let variant_type = self.next()?;

		match variant_type {
			NONUNIT_VARIANT => visitor.visit_enum(Enum::new(self)),
			STRUCT_VARIANT => visitor.visit_enum(Enum::new(self)),
			UNIT_VARIANT => {
				let variant_index: u32 = self.next_u32()?;
				let variant: &'de str = variants[variant_index as usize];

				visitor.visit_enum(variant.into_deserializer())
			}
			_ => Err(BinaryError::MissingOrInvalidFlag {
				actual: variant_type,
				expected: 0xFE,
			}),
		}
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_u32(self.next_u32()?)
	}

	fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}

	fn deserialize_any<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}
}

struct BinarySeries<'a, 'de: 'a> {
	de: &'a mut Deserializer<'de>,
	len: usize,
	position: usize,
}

impl<'a, 'de> BinarySeries<'a, 'de> {
	pub fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
		Self {
			de,
			len,
			position: 0,
		}
	}
}

impl<'de, 'a> SeqAccess<'de> for BinarySeries<'a, 'de> {
	type Error = BinaryError;

	fn next_element_seed<T>(
		&mut self,
		seed: T,
	) -> std::result::Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		self.position += 1;
		if self.position == self.len + 1 {
			return Ok(None);
		} else if self.position > self.len {
			return Err(BinaryError::InvalidLength {
				actual: self.position,
				expected: self.len,
			});
		}
		seed.deserialize(&mut *self.de).map(Some)
	}
}

impl<'de, 'a> MapAccess<'de> for BinarySeries<'a, 'de> {
	type Error = BinaryError;

	fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
	where
		K: de::DeserializeSeed<'de>,
	{
		self.position += 1;
		if self.position == self.len + 1 {
			return Ok(None);
		} else if self.position > self.len {
			return Err(BinaryError::InvalidLength {
				actual: self.position,
				expected: self.len,
			});
		}
		seed.deserialize(&mut *self.de).map(Some)
	}

	fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: de::DeserializeSeed<'de>,
	{
		seed.deserialize(&mut *self.de)
	}
}

struct Enum<'a, 'de: 'a> {
	de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
	fn new(de: &'a mut Deserializer<'de>) -> Self {
		Enum { de }
	}
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
	type Error = BinaryError;
	type Variant = Self;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
	where
		V: DeserializeSeed<'de>,
	{
		Ok((seed.deserialize(&mut *self.de)?, self))
	}
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
	type Error = BinaryError;

	fn unit_variant(self) -> Result<()> {
		Err(BinaryError::UnexpectedType)
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
	where
		T: DeserializeSeed<'de>,
	{
		seed.deserialize(self.de)
	}

	fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		de::Deserializer::deserialize_seq(self.de, visitor)
	}

	fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		de::Deserializer::deserialize_seq(self.de, visitor)
	}
}
