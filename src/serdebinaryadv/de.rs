use num::traits::FromBytes;
use serde::Deserialize;
use serde::de::{
	self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
	Visitor,
};

use crate::{Endianness, Options};

use super::error::{Error, Result};

/// deserializes a Vec<u8> into Rust structures
pub struct Deserializer<'de> {
	input: &'de Vec<u8>,
	offset: usize,
	options: Options,
}

impl<'de> Deserializer<'de> {
	pub fn from_bytes(input: &'de Vec<u8>, options: Options) -> Self {
		Deserializer {
			input,
			offset: 0,
			options,
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

impl<'de> Deserializer<'de> {
	fn peek(&mut self) -> Result<u8> {
		if self.input.len() < self.offset + 1 {
			Err(Error::Eof)
		} else {
			Ok(self.input[self.offset + 1])
		}
	}

	fn next(&mut self) -> Result<u8> {
		if self.input.len() == 0 {
			return Err(Error::Eof);
		}
		self.offset += 1;
		let v = self.input[self.offset];
		Ok(v)
	}

	fn take(&mut self, len: usize) -> Result<Vec<u8>> {
		if self.input.len() < (len - 1) {
			return Err(Error::Eof);
		}
		let v = self.input[0..len].to_vec();
		self.offset += len;
		Ok(v)
	}

	fn deserialize_num<T: FromBytes>(self, v: Vec<u8>) -> T {
		match self.options.endianness {
			Endianness::Native => T::from_ne_bytes(v),
			Endianness::Little => T::from_le_bytes(v.try_into().unwrap()),
			Endianness::Big => T::from_be_bytes(v.try_into().unwrap()),
		}
	}
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
	type Error = Error;

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

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let bytes = self.take(2).unwrap();
		visitor.visit_i16(match self.options.endianness {
			Endianness::Native => i16::from_ne_bytes(bytes.try_into().unwrap()),
			Endianness::Little => i16::from_le_bytes(bytes.try_into().unwrap()),
			Endianness::Big => i16::from_be_bytes(bytes.try_into().unwrap()),
		})
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_i32(self.parse_signed()?)
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_i64(self.parse_signed()?)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_u8(self.parse_unsigned()?)
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_u16(self.parse_unsigned()?)
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_u32(self.parse_unsigned()?)
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_u64(self.parse_unsigned()?)
	}

	// Float parsing is stupidly hard.
	fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}

	// Float parsing is stupidly hard.
	fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}

	// The `Serializer` implementation on the previous page serialized chars as
	// single-character strings so handle that representation here.
	fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		// Parse a string, check that it is one character, call `visit_char`.
		unimplemented!()
	}

	// Refer to the "Understanding deserializer lifetimes" page for information
	// about the three deserialization flavors of strings in Serde.
	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_borrowed_str(self.parse_string()?)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	// The `Serializer` implementation on the previous page serialized byte
	// arrays as JSON arrays of bytes. Handle that representation here.
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

	// An absent optional is represented as the JSON `null` and a present
	// optional is represented as just the contained value.
	//
	// As commented in `Serializer` implementation, this is a lossy
	// representation. For example the values `Some(())` and `None` both
	// serialize as just `null`. Unfortunately this is typically what people
	// expect when working with JSON. Other formats are encouraged to behave
	// more intelligently if possible.
	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		if self.input.starts_with("null") {
			self.input = &self.input["null".len()..];
			visitor.visit_none()
		} else {
			visitor.visit_some(self)
		}
	}

	// In Serde, unit means an anonymous value containing no data.
	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		if self.input.starts_with("null") {
			self.input = &self.input["null".len()..];
			visitor.visit_unit()
		} else {
			Err(Error::ExpectedNull)
		}
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
	fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	// Deserialization of compound types like sequences and maps happens by
	// passing the visitor an "Access" object that gives it the ability to
	// iterate through the data contained in the sequence.
	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		// Parse the opening bracket of the sequence.
		if self.next_char()? == '[' {
			// Give the visitor access to each element of the sequence.
			let value = visitor.visit_seq(CommaSeparated::new(self))?;
			// Parse the closing bracket of the sequence.
			if self.next_char()? == ']' {
				Ok(value)
			} else {
				Err(Error::ExpectedArrayEnd)
			}
		} else {
			Err(Error::ExpectedArray)
		}
	}

	// Tuples look just like sequences in JSON. Some formats may be able to
	// represent tuples more efficiently.
	//
	// As indicated by the length parameter, the `Deserialize` implementation
	// for a tuple in the Serde data model is required to know the length of the
	// tuple before even looking at the input data.
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
