use crate::{NONE_FLAG, SOME_FLAG};

use super::error::{BinaryError, Result};
use serde::de::{self, Visitor};
use serde::{Deserialize, de::value::SeqDeserializer};

macro_rules! impl_deserialize_num {
	($name:ident, $ty:ty, $visit:ident) => {
		fn $name<V>(self, visitor: V) -> Result<V::Value>
		where
			V: Visitor<'de>,
		{
			let bytes: Vec<u8> = self.take(size_of::<$ty>()).unwrap();

			let value = match self.options.endianness {
				Endianness::Native => <$ty>::from_ne_bytes(bytes.try_into().unwrap()),
				Endianness::Little => <$ty>::from_le_bytes(bytes.try_into().unwrap()),
				Endianness::Big => <$ty>::from_be_bytes(bytes.try_into().unwrap()),
			};

			visitor.$visit(value)
		}
	};
}

macro_rules! impl_take_uxx {
	($name:ident, $ty:ty) => {
		fn $name(&mut self) -> Result<$ty> {
			let bytes = self.take(size_of::<$ty>()).unwrap();
			match self.options.endianness {
				Endianness::Native => Ok(<$ty>::from_ne_bytes(bytes.try_into().unwrap())),
				Endianness::Little => Ok(<$ty>::from_le_bytes(bytes.try_into().unwrap())),
				Endianness::Big => Ok(<$ty>::from_be_bytes(bytes.try_into().unwrap())),
			}
		}
	};
}

/// deserializes a Vec<u8> into Rust structures
pub struct Deserializer<'de> {
	data: Vec<u8>,
	options: Options,
	_flag: &'de bool,
}

impl<'de> Deserializer<'de> {
	pub fn from_bytes<'a, T>(s: Vec<u8>, options: Options) -> Result<T>
	where
		T: Deserialize<'a>,
	{
		let mut deserializer = Deserializer::new_from_bytes(s, options);

		let t = T::deserialize(&mut deserializer)?;
		Ok(t)
	}

	fn new_from_bytes(input: Vec<u8>, options: Options) -> Deserializer<'de> {
		Deserializer {
			data: input,
			options,
			_flag: &true,
		}
	}

	fn peek(&mut self) -> Result<u8> {
		if self.data.len() == 0 {
			Err(BinaryError::Eof)
		} else {
			Ok(self.data[0])
		}
	}

	fn next(&mut self) -> Result<u8> {
		if self.data.len() == 0 {
			Err(BinaryError::Eof)
		} else {
			Ok(self.data.pop().unwrap())
		}
	}

	fn take(&mut self, len: usize) -> Result<Vec<u8>> {
		if self.data.len() < len {
			Err(BinaryError::Eof)
		} else {
			let working = self.data.clone();
			let (res, rem) = working.split_at(len);
			self.data = rem.to_vec();
			Ok(res.to_vec())
		}
	}

	impl_take_uxx!(take_u16, u16);

	impl_take_uxx!(take_u32, u32);

	impl_take_uxx!(take_usize, usize);
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
	type Error = BinaryError;

	impl_deserialize_num!(deserialize_i16, i16, visit_i16);
	impl_deserialize_num!(deserialize_i32, i32, visit_i32);
	impl_deserialize_num!(deserialize_i64, i64, visit_i64);
	impl_deserialize_num!(deserialize_u16, u16, visit_u16);
	impl_deserialize_num!(deserialize_u32, u32, visit_u32);
	impl_deserialize_num!(deserialize_u64, u64, visit_u64);
	impl_deserialize_num!(deserialize_f32, f32, visit_f32);
	impl_deserialize_num!(deserialize_f64, f64, visit_f64);

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_bool(self.next().unwrap() == 0x00)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_i8(self.next().unwrap() as i8)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_u8(self.next().unwrap())
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let b1 = self.next().unwrap();
		if b1 <= 0x7F {
			visitor.visit_char(char::from(b1))
		} else if b1 >= 0xC0 && b1 <= 0xDF {
			visitor.visit_char(
				String::from_utf8(vec![b1, self.next().unwrap()])
					.unwrap()
					.chars()
					.next()
					.unwrap(),
			)
		} else if b1 >= 0xE0 && b1 <= 0xEF {
			visitor.visit_char(
				String::from_utf8(vec![b1, self.next().unwrap(), self.next().unwrap()])
					.unwrap()
					.chars()
					.next()
					.unwrap(),
			)
		} else if b1 >= 0xF0 {
			visitor.visit_char(
				String::from_utf8(vec![
					b1,
					self.next().unwrap(),
					self.next().unwrap(),
					self.next().unwrap(),
				])
				.unwrap()
				.chars()
				.next()
				.unwrap(),
			)
		} else {
			return Err(BinaryError::InvalidUnicode);
		}
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let size = self.take_usize().unwrap();

		let mut s: String = String::new();
		let data: Vec<u8> = self.take(size).unwrap();
		s.push_str(String::from_utf8(data).unwrap().as_str());
		visitor.visit_str(&s)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let len = self.take_usize().unwrap();
		let bytes = self.take(len).unwrap();
		visitor.visit_bytes(&bytes)
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_bytes(visitor)
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let flag: u8 = self.next().unwrap();
		if flag == NONE_FLAG {
			visitor.visit_none()
		} else if flag == SOME_FLAG {
			visitor.visit_some(self)
		} else {
			Err(BinaryError::InvalidOptionFlag)
		}
	}

	// In Serde, unit means an anonymous value containing no data.
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
		self.deserialize_unit(visitor)
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
		let len: usize = self.take_usize().unwrap();
		visitor.visit_seq(SeqDeserializer::new(
			self.take(len).unwrap().iter().map(|c| *c),
		))
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
		self.deserialize_seq(visitor)
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
		self.deserialize_map(visitor)
	}

	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_u32::<BinaryError>(self.take_u32().unwrap())
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_str(visitor)
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
