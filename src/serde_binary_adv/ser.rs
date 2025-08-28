//! Serialize a Rust structure into binary data.

use crate::serde_binary_adv::common::{
	compress_usize,
	flags::{self, STRUCT, STRUCT_VARIANT, UNIT_VARIANT},
};

use super::BinaryError;
use super::Result;
use num::traits::ToBytes;
use serde::{Serialize, ser};

/// A structure for serializing Rust values into binary.
pub struct Serializer {
	output: Vec<u8>,
	big_endian: bool,
}

impl Serializer {
	/// Converts a Rust value into a binary representation and returns a Vec<u8> of the bytes
	pub fn to_bytes<T>(value: &T, big_endian: bool) -> Result<Vec<u8>>
	where
		T: Serialize,
	{
		let mut serializer = Self::new(big_endian);
		value.serialize(&mut serializer)?;
		Ok(serializer.output)
	}

	/// Creates a new binary Serializer
	pub fn new(big_endian: bool) -> Self {
		Self {
			output: Vec::new(),
			big_endian,
		}
	}

	fn serialize_num<T: ToBytes>(self: &mut Self, v: T) -> Result<()> {
		if self.big_endian {
			self.output.append(&mut v.to_be_bytes().as_mut().to_vec());
		} else {
			self.output.append(&mut v.to_le_bytes().as_mut().to_vec());
		}
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
	type Error = BinaryError;

	type SerializeSeq = Self;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;
	type SerializeMap = Self;
	type SerializeStruct = Self;
	type SerializeStructVariant = Self;

	fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
		self.serialize_u8(if v { 1 } else { 0 })
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

	fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
		self.serialize_num(v)
	}

	fn serialize_char(self, v: char) -> Result<Self::Ok> {
		let mut buf: [u8; 4] = [0, 0, 0, 0];
		self.serialize_vec(v.encode_utf8(&mut buf).as_bytes().to_vec())
	}

	fn serialize_str(self, v: &str) -> Result<Self::Ok> {
		self.serialize_vec(compress_usize(v.bytes().len())).unwrap();
		self.serialize_vec(v.as_bytes().to_vec())
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
		self.serialize_vec(compress_usize(v.len())).unwrap();
		self.serialize_vec(v.to_vec()).unwrap();
		Ok(())
	}

	fn serialize_none(self) -> Result<Self::Ok> {
		self.serialize_u8(flags::NONE)
	}

	fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
	where
		T: ?Sized + ser::Serialize,
	{
		self.serialize_u8(flags::SOME).unwrap();
		value.serialize(self)
	}

	fn serialize_unit(self) -> Result<Self::Ok> {
		Ok(())
	}

	fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
		Ok(())
	}

	fn serialize_unit_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		_variant: &'static str,
	) -> Result<Self::Ok> {
		self.serialize_u8(UNIT_VARIANT).unwrap();
		variant_index.serialize(&mut *self)
	}

	fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
	where
		T: ?Sized + ser::Serialize,
	{
		value.serialize(self)
	}

	fn serialize_newtype_variant<T>(
		self,
		_name: &'static str,
		variant_index: u32,
		_variant: &'static str,
		value: &T,
	) -> Result<Self::Ok>
	where
		T: ?Sized + ser::Serialize,
	{
		variant_index.serialize(&mut *self).unwrap();
		value.serialize(self)
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
		match len {
			Some(n) => {
				self.serialize_vec(compress_usize(n)).unwrap();
				Ok(self)
			}
			// Serializing sequences of unknown length to binary is difficult, since any value that
			// can be used to mark the end of the sequence can also be a member
			None => unimplemented!(),
		}
	}

	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
		self.serialize_seq(Some(len))
	}

	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct> {
		self.serialize_seq(Some(len))
	}

	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		_variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant> {
		variant_index.serialize(&mut *self).unwrap();
		self.serialize_vec(compress_usize(len)).unwrap();
		Ok(self)
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
		match len {
			Some(n) => {
				self.serialize_vec(compress_usize(n)).unwrap();
				Ok(self)
			}
			// Serializing maps of unknown length to binary is difficult, since any value that
			// can be used to mark the end of the sequence can also be a member
			None => unimplemented!(),
		}
	}

	fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
		self.output.push(STRUCT);
		name.serialize(&mut *self).unwrap();
		self.serialize_vec(compress_usize(len)).unwrap();
		Ok(self)
	}

	fn serialize_struct_variant(
		self,
		name: &'static str,
		variant_index: u32,
		_variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant> {
		self.output.push(STRUCT_VARIANT);
		name.serialize(&mut *self).unwrap();
		variant_index.serialize(&mut *self).unwrap();
		self.serialize_vec(compress_usize(len)).unwrap();
		Ok(self)
	}
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
	type Ok = ();
	type Error = BinaryError;

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
	type Error = BinaryError;

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
	type Error = BinaryError;

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
	type Error = BinaryError;

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
	type Error = BinaryError;

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
	type Error = BinaryError;

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
	type Error = BinaryError;

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
