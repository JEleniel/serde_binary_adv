use std::num;

use crate::{Endianness, Options};

use super::error::{Error, Result};
use ::num::traits::ToBytes;
use serde::ser;

pub struct Serializer {
	output: Vec<u8>,
	endianness: Endianness,
}

pub fn to_bytes<T>(value: &T, options: Options) -> super::error::Result<Vec<u8>> {
	let mut serializer = Serializer {
		output: vec![],
		endianness: options.endianness,
	};
	value.serialize(&mut serializer)?;
	Ok(serializer.output)
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
		self.output.push(if v { 1 } else { 0 });
		Ok(())
	}

	fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
		let be_binding = &mut v.to_be_bytes().to_vec();
		let le_binding = &mut v.to_le_bytes().to_vec();
		let ne_binding = &mut v.to_ne_bytes().to_vec();

		self.output.append(match self.endianness {
			Endianness::Big => be_binding,
			Endianness::Little => le_binding,
			Endianness::Native => ne_binding,
		});
		Ok(())
	}

	fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
		let be_binding = &mut v.to_be_bytes().to_vec();
		let le_binding = &mut v.to_le_bytes().to_vec();
		let ne_binding = &mut v.to_ne_bytes().to_vec();

		self.output.append(match self.endianness {
			Endianness::Big => be_binding,
			Endianness::Little => le_binding,
			Endianness::Native => ne_binding,
		});
		Ok(())
	}

	fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
		let be_binding = &mut v.to_be_bytes().to_vec();
		let le_binding = &mut v.to_le_bytes().to_vec();
		let ne_binding = &mut v.to_ne_bytes().to_vec();

		self.output.append(match self.endianness {
			Endianness::Big => be_binding,
			Endianness::Little => le_binding,
			Endianness::Native => ne_binding,
		});
		Ok(())
	}

	fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
		let be_binding = &mut v.to_be_bytes().to_vec();
		let le_binding = &mut v.to_le_bytes().to_vec();
		let ne_binding = &mut v.to_ne_bytes().to_vec();

		self.output.append(match self.endianness {
			Endianness::Big => be_binding,
			Endianness::Little => le_binding,
			Endianness::Native => ne_binding,
		});
		Ok(())
	}

	fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
		let be_binding = &mut v.to_be_bytes().to_vec();
		let le_binding = &mut v.to_le_bytes().to_vec();
		let ne_binding = &mut v.to_ne_bytes().to_vec();

		self.output.append(match self.endianness {
			Endianness::Big => be_binding,
			Endianness::Little => le_binding,
			Endianness::Native => ne_binding,
		});
		Ok(())
	}

	fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
		let be_binding = &mut v.to_be_bytes().to_vec();
		let le_binding = &mut v.to_le_bytes().to_vec();
		let ne_binding = &mut v.to_ne_bytes().to_vec();

		self.output.append(match self.endianness {
			Endianness::Big => be_binding,
			Endianness::Little => le_binding,
			Endianness::Native => ne_binding,
		});
		Ok(())
	}

	fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
		let be_binding = &mut v.to_be_bytes().to_vec();
		let le_binding = &mut v.to_le_bytes().to_vec();
		let ne_binding = &mut v.to_ne_bytes().to_vec();

		self.output.append(match self.endianness {
			Endianness::Big => be_binding,
			Endianness::Little => le_binding,
			Endianness::Native => ne_binding,
		});
		Ok(())
	}

	fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
		let be_binding = &mut v.to_be_bytes().to_vec();
		let le_binding = &mut v.to_le_bytes().to_vec();
		let ne_binding = &mut v.to_ne_bytes().to_vec();

		self.output.append(match self.endianness {
			Endianness::Big => be_binding,
			Endianness::Little => le_binding,
			Endianness::Native => ne_binding,
		});
		Ok(())
	}

	fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
		let be_binding = &mut v.to_be_bytes().to_vec();
		let le_binding = &mut v.to_le_bytes().to_vec();
		let ne_binding = &mut v.to_ne_bytes().to_vec();

		self.output.append(match self.endianness {
			Endianness::Big => be_binding,
			Endianness::Little => le_binding,
			Endianness::Native => ne_binding,
		});
		Ok(())
	}

	fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
		let be_binding = &mut v.to_be_bytes().to_vec();
		let le_binding = &mut v.to_le_bytes().to_vec();
		let ne_binding = &mut v.to_ne_bytes().to_vec();

		self.output.append(match self.endianness {
			Endianness::Big => be_binding,
			Endianness::Little => le_binding,
			Endianness::Native => ne_binding,
		});
		Ok(())
	}

	fn serialize_char(self, v: char) -> Result<Self::Ok> {
		todo!()
	}

	fn serialize_str(self, v: &str) -> Result<Self::Ok> {
		todo!()
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
		self.output.append(&mut v.to_vec());
		Ok(())
	}

	fn serialize_none(self) -> Result<Self::Ok> {
		self.output.push(0x00);
		Ok(())
	}

	fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
	where
		T: ?Sized + ser::Serialize,
	{
		todo!()
	}

	fn serialize_unit(self) -> Result<Self::Ok> {
		todo!()
	}

	fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
		todo!()
	}

	fn serialize_unit_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
	) -> Result<Self::Ok> {
		todo!()
	}

	fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok>
	where
		T: ?Sized + ser::Serialize,
	{
		todo!()
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
		todo!()
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
		todo!()
	}

	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
		todo!()
	}

	fn serialize_tuple_struct(
		self,
		name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct> {
		todo!()
	}

	fn serialize_tuple_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant> {
		todo!()
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
		todo!()
	}

	fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
		todo!()
	}

	fn serialize_struct_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant> {
		todo!()
	}
}
