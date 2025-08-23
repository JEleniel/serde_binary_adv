use super::error::{Error, Result};
use crate::{CharacterEncoding, Endianness, Options, StringType};
use num::traits::ToBytes;
use serde::{Serialize, ser};

/// serializes a Rust struct into raw bytes according to the provided options
pub struct Serializer {
	output: Vec<u8>,
	options: Options,
}

/// convert a Rust struct to raw bytes according to the options specified
pub fn to_bytes<T>(value: &T, options: Options) -> super::error::Result<Vec<u8>>
where
	T: Serialize,
{
	let mut serializer = Serializer {
		output: vec![],
		options,
	};
	value.serialize(&mut serializer)?;
	Ok(serializer.output)
}

impl Serializer {
	fn serialize_num<T: ToBytes>(self: &mut Self, v: T) -> Result<()> {
		let ne_binding = &mut v.to_ne_bytes().as_mut().to_vec();
		let le_binding = &mut v.to_le_bytes().as_mut().to_vec();
		let be_binding = &mut v.to_be_bytes().as_mut().to_vec();

		self.output.append(match self.options.endianness {
			Endianness::Native => ne_binding,
			Endianness::Little => le_binding,
			Endianness::Big => be_binding,
		});
		Ok(())
	}

	fn serialize_vec<T: ToBytes>(self: &mut Self, v: Vec<T>) -> Result<()> {
		for item in v {
			self.serialize_num(item).unwrap()
		}
		Ok(())
	}
}

impl<'a> ser::Serializer for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	type SerializeSeq = Self;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;
	type SerializeMap = Self;
	type SerializeStruct = Self;
	type SerializeStructVariant = Self;

	fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
		self.serialize_num(if v { 1 } else { 0 })
	}

	fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_char(self, v: char) -> Result<Self::Ok> {
		if self.options.character_encoding == CharacterEncoding::ASCII && !v.is_ascii() {
			return Err(Error::InvalidASCII);
		}
		match self.options.character_encoding {
			CharacterEncoding::ASCII | CharacterEncoding::UTF8 => {
				let mut buf: [u8; 4] = [0, 0, 0, 0];
				self.serialize_vec(v.encode_utf8(&mut buf).as_bytes().to_vec())
			}
			CharacterEncoding::UTF16 => {
				let mut buf: [u16; 2] = [0, 0];
				let nbuf = v.encode_utf16(&mut buf).to_vec();
				self.serialize_vec(nbuf)
			}
		}
	}

	fn serialize_str(self, v: &str) -> Result<Self::Ok> {
		if self.options.character_encoding == CharacterEncoding::ASCII && !v.is_ascii() {
			return Err(Error::InvalidASCII);
		}
		// Write the length (bytes) header if needed
		match self.options.string_type {
			StringType::NullTerminated | StringType::FixedLen => {}
			_ => {
				self.serialize_num(v.bytes().len()).unwrap();
			}
		}
		match self.options.character_encoding {
			CharacterEncoding::ASCII | CharacterEncoding::UTF8 => {
				self.serialize_vec(v.as_bytes().to_vec()).unwrap();
			}
			CharacterEncoding::UTF16 => {
				self.serialize_vec(v.encode_utf16().collect()).unwrap();
			}
		}
		// Write the null ternimator, if required
		match self.options.string_type {
			StringType::NullTerminated | StringType::SizeTaggedAndNullTerminated => {
				self.serialize_num(0).unwrap();
			}
			_ => {}
		}
		Ok(())
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
		v.serialize(&mut *self)
	}

	fn serialize_none(self) -> Result<Self::Ok> {
		self.serialize_num(0)
	}

	fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
	where
		T: ?Sized + ser::Serialize,
	{
		if self.options.self_describing {
			self.serialize_num(0xFF).unwrap();
		}
		value.serialize(self)
	}

	fn serialize_unit(self) -> Result<Self::Ok> {
		Ok(())
	}

	fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
		if self.options.self_describing {
			self.serialize_str(name)
		} else {
			Ok(())
		}
	}

	fn serialize_unit_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
	) -> Result<Self::Ok> {
		if self.options.self_describing {
			name.serialize(&mut *self).unwrap();
		}
		variant_index.serialize(&mut *self).unwrap();
		if self.options.self_describing {
			variant.serialize(&mut *self).unwrap();
		}
		Ok(())
	}

	fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok>
	where
		T: ?Sized + ser::Serialize,
	{
		if self.options.self_describing {
			name.serialize(&mut *self).unwrap();
		}
		value.serialize(self)
	}

	fn serialize_newtype_variant<T>(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<Self::Ok>
	where
		T: ?Sized + ser::Serialize,
	{
		if self.options.self_describing {
			name.serialize(&mut *self).unwrap();
		}
		variant_index.serialize(&mut *self).unwrap();
		if self.options.self_describing {
			variant.serialize(&mut *self);
		}
		value.serialize(self)
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
		match len {
			Some(n) => {
				self.serialize_num(n).unwrap();
				Ok(self)
			}
			None => unimplemented!(),
		}
	}

	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
		len.serialize(&mut *self).unwrap();
		Ok(self)
	}

	fn serialize_tuple_struct(
		self,
		name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct> {
		if self.options.self_describing {
			name.serialize(&mut *self).unwrap();
		}
		len.serialize(&mut *self).unwrap();
		Ok(self)
	}

	fn serialize_tuple_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant> {
		if self.options.self_describing {
			name.serialize(&mut *self).unwrap();
		}
		variant_index.serialize(&mut *self).unwrap();
		if self.options.self_describing {
			variant.serialize(&mut *self).unwrap();
		}
		len.serialize(&mut *self).unwrap();
		Ok(self)
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
		match len {
			Some(n) => {
				n.serialize(&mut *self).unwrap();
				Ok(self)
			}
			None => unimplemented!(),
		}
	}

	fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
		if self.options.self_describing {
			self.output.push(0xFE);
			name.serialize(&mut *self).unwrap();
			len.serialize(&mut *self).unwrap();
		}
		Ok(self)
	}

	fn serialize_struct_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant> {
		if self.options.self_describing {
			self.output.push(0xFD);
			name.serialize(&mut *self).unwrap();
		}
		variant_index.serialize(&mut *self).unwrap();
		if self.options.self_describing {
			variant.serialize(&mut *self).unwrap();
			len.serialize(&mut *self).unwrap();
		}
		Ok(self)
	}
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	// Close the sequence.
	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	fn serialize_key<T>(&mut self, key: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		key.serialize(&mut **self)
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}
