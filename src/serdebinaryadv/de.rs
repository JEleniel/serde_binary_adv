use super::error::{Error, Result};
use crate::{CharacterEncoding, Endianness, Options, StringType, serdebinaryadv::bytes::Raw};
use serde::Deserialize;
use serde::de::{
	self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
	Visitor,
};

macro_rules! impl_deserialize_num {
	($name:ident, $ty:ty, $visit:ident) => {
		fn $name<V>(self, visitor: V) -> Result<V::Value>
		where
			V: Visitor<'de>,
		{
			let bytes = self.raw.take(size_of::<$ty>()).unwrap();

			let value = match self.options.endianness {
				Endianness::Native => <$ty>::from_ne_bytes(bytes.try_into().unwrap()),
				Endianness::Little => <$ty>::from_le_bytes(bytes.try_into().unwrap()),
				Endianness::Big => <$ty>::from_be_bytes(bytes.try_into().unwrap()),
			};

			visitor.$visit(value)
		}
	};
}

/// deserializes a Vec<u8> into Rust structures
pub struct Deserializer<'de> {
	raw: Raw,
	options: Options,
	flag: &'de bool,
}

impl<'de> Deserializer<'de> {
	fn from_bytes(input: &'de Vec<u8>, options: Options) -> Deserializer<'de> {
		Deserializer {
			raw: Raw::from(input),
			options,
			flag: &true,
		}
	}
}

pub fn from_bytes<'a, T>(s: &'a Vec<u8>, options: Options) -> Result<T>
where
	T: Deserialize<'a>,
{
	let mut deserializer = Deserializer::from_bytes(s, options);

	let t = T::deserialize(&mut deserializer)?;
	Ok(t)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
	type Error = Error;

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
		visitor.visit_bool(self.raw.next().unwrap() == 0x00)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_i8(self.raw.next().unwrap() as i8)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_u8(self.raw.next().unwrap())
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let mut s: String = String::new();

		match self.options.character_encoding {
			CharacterEncoding::ASCII => {
				s = String::from_utf8(self.raw.take(1).unwrap()).unwrap();
			}
			CharacterEncoding::UTF8 => {
				let b1 = self.raw.peek().unwrap();
				if b1 <= 0x7F {
					s = String::from_utf8(self.raw.take(1).unwrap()).unwrap();
				} else if b1 >= 0x80 && b1 <= 0xBF {
					s = String::from_utf8(self.raw.take(2).unwrap()).unwrap();
				} else if b1 >= 0xE0 && b1 <= 0xEF {
					s = String::from_utf8(self.raw.take(3).unwrap()).unwrap();
				} else if b1 >= 0xF0 {
					s = String::from_utf8(self.raw.take(4).unwrap()).unwrap();
				} else {
					return Err(Error::InvalidUnicode);
				}
			}
			CharacterEncoding::UTF16 => {
				let b1: u16 = self.raw.take_u16(&self.options.endianness).unwrap();

				if b1 <= 0xD7FF || (b1 >= 0xE000 && b1 <= 0xFFFF) {
					s = String::from_utf16(&[b1]).unwrap();
				} else {
					s = String::from_utf16(&[
						b1,
						self.raw.take_u16(&self.options.endianness).unwrap(),
					])
					.unwrap();
				}
			}
		}
		visitor.visit_char(s.chars().next().unwrap())
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let mut size: usize = 0;
		if self.options.string_type == StringType::SizeTagged
			|| self.options.string_type == StringType::SizeTaggedAndNullTerminated
		{
			size = self.raw.take_usize(&self.options.endianness).unwrap();
		}

		let mut s: String = String::new();
		match self.options.character_encoding {
			CharacterEncoding::ASCII | CharacterEncoding::UTF8 => {
				let mut data: Vec<u8> = vec![];
				if size > 0 {
					data = self.raw.take(size).unwrap()
				} else {
					while self.raw.peek().unwrap() != 0x00 {
						data.push(self.raw.next().unwrap());
					}
				}
				s.push_str(String::from_utf8(data).unwrap().as_str());
			}
			CharacterEncoding::UTF16 => {
				let mut data: Vec<u16> = vec![];
				if size > 0 {
					for _n in 1..size / 2 {
						data.push(self.raw.take_u16(&self.options.endianness).unwrap());
					}
				}
				s.push_str(String::from_utf16(&data).unwrap().as_str());
			}
		}
		if self.options.string_type == StringType::NullTerminated
			|| self.options.string_type == StringType::SizeTaggedAndNullTerminated
		{
			if self.raw.peek().unwrap() != 0x00 {
				return Err(Error::MissingStringTerminator);
			}
		}
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
		let len: usize = self.raw.take_usize(&self.options.endianness).unwrap();
		visitor.visit_bytes(self.raw.take(len).unwrap().as_slice())
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
		let flag: u8 = self.raw.next().unwrap();
		if flag == 0x00 {
			visitor.visit_none()
		} else if flag == 0xFF {
			visitor.visit_some(self)
		} else {
			Err(Error::InvalidOptionFlag)
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

	// As is done here, serializers are encouraged to treat newtype structs as
	// insignificant wrappers around the data they contain. That means not
	// parsing anything other than the contained value.
	fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}

	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	// Tuple structs look just like sequences in JSON.
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

	// Much like `deserialize_seq` but calls the visitors `visit_map` method
	// with a `MapAccess` implementation, rather than the visitor's `visit_seq`
	// method with a `SeqAccess` implementation.
	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		// Parse the opening brace of the map.
		if self.next_char()? == '{' {
			// Give the visitor access to each entry of the map.
			let value = visitor.visit_map(CommaSeparated::new(self))?;
			// Parse the closing brace of the map.
			if self.next_char()? == '}' {
				Ok(value)
			} else {
				Err(Error::ExpectedMapEnd)
			}
		} else {
			Err(Error::ExpectedMap)
		}
	}

	// Structs look just like maps in JSON.
	//
	// Notice the `fields` parameter - a "struct" in the Serde data model means
	// that the `Deserialize` implementation is required to know what the fields
	// are before even looking at the input data. Any key-value pairing in which
	// the fields cannot be known ahead of time is probably a map.
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
		if self.peek_char()? == '"' {
			// Visit a unit variant.
			visitor.visit_enum(self.parse_string()?.into_deserializer())
		} else if self.next_char()? == '{' {
			// Visit a newtype variant, tuple variant, or struct variant.
			let value = visitor.visit_enum(Enum::new(self))?;
			// Parse the matching close brace.
			if self.next_char()? == '}' {
				Ok(value)
			} else {
				Err(Error::ExpectedMapEnd)
			}
		} else {
			Err(Error::ExpectedEnum)
		}
	}

	// An identifier in Serde is the type that identifies a field of a struct or
	// the variant of an enum. In JSON, struct fields and enum variants are
	// represented as strings. In other formats they may be represented as
	// numeric indices.
	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	// Like `deserialize_any` but indicates to the `Deserializer` that it makes
	// no difference which `Visitor` method is called because the data is
	// ignored.
	//
	// Some deserializers are able to implement this more efficiently than
	// `deserialize_any`, for example by rapidly skipping over matched
	// delimiters without paying close attention to the data in between.
	//
	// Some formats are not able to implement this at all. Formats that can
	// implement `deserialize_any` and `deserialize_ignored_any` are known as
	// self-describing.
	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_any(visitor)
	}

	/// not implemented because raw binary formats are, by definition, not self describing
	fn deserialize_any<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}
}

// In order to handle commas correctly when deserializing a JSON array or map,
// we need to track whether we are on the first element or past the first
// element.
struct CommaSeparated<'a, 'de: 'a> {
	de: &'a mut Deserializer<'de>,
	first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
	fn new(de: &'a mut Deserializer<'de>) -> Self {
		CommaSeparated { de, first: true }
	}
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
	where
		T: DeserializeSeed<'de>,
	{
		// Check if there are no more elements.
		if self.de.peek_char()? == ']' {
			return Ok(None);
		}
		// Comma is required before every element except the first.
		if !self.first && self.de.next_char()? != ',' {
			return Err(Error::ExpectedArrayComma);
		}
		self.first = false;
		// Deserialize an array element.
		seed.deserialize(&mut *self.de).map(Some)
	}
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
	where
		K: DeserializeSeed<'de>,
	{
		// Check if there are no more entries.
		if self.de.peek_char()? == '}' {
			return Ok(None);
		}
		// Comma is required before every entry except the first.
		if !self.first && self.de.next_char()? != ',' {
			return Err(Error::ExpectedMapComma);
		}
		self.first = false;
		// Deserialize a map key.
		seed.deserialize(&mut *self.de).map(Some)
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
	where
		V: DeserializeSeed<'de>,
	{
		// It doesn't make a difference whether the colon is parsed at the end
		// of `next_key_seed` or at the beginning of `next_value_seed`. In this
		// case the code is a bit simpler having it here.
		if self.de.next_char()? != ':' {
			return Err(Error::ExpectedMapColon);
		}
		// Deserialize a map value.
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

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
	type Error = Error;
	type Variant = Self;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
	where
		V: DeserializeSeed<'de>,
	{
		// The `deserialize_enum` method parsed a `{` character so we are
		// currently inside of a map. The seed will be deserializing itself from
		// the key of the map.
		let val = seed.deserialize(&mut *self.de)?;
		// Parse the colon separating map key from value.
		if self.de.next_char()? == ':' {
			Ok((val, self))
		} else {
			Err(Error::ExpectedMapColon)
		}
	}
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
	type Error = Error;

	// If the `Visitor` expected this variant to be a unit variant, the input
	// should have been the plain string case handled in `deserialize_enum`.
	fn unit_variant(self) -> Result<()> {
		Err(Error::ExpectedString)
	}

	// Newtype variants are represented in JSON as `{ NAME: VALUE }` so
	// deserialize the value here.
	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
	where
		T: DeserializeSeed<'de>,
	{
		seed.deserialize(self.de)
	}

	// Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
	// deserialize the sequence of data here.
	fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		de::Deserializer::deserialize_seq(self.de, visitor)
	}

	// Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
	// deserialize the inner map here.
	fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		de::Deserializer::deserialize_map(self.de, visitor)
	}
}
