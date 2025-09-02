mod binaryerror;
mod common;
mod de;
mod ser;
#[cfg(feature = "streaming")]
pub mod stream;

pub use binaryerror::BinaryError;
pub use common::{ByteFormat, Result};
pub use de::Deserializer;
pub use ser::Serializer;

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use serde::{Deserialize, Serialize};

	use crate::{Deserializer, Serializer};

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	struct Unit;

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	struct NewType(u8);

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	struct TupleStruct(u8, u8, u8);

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	struct Test {
		pub byte: u8,
		pub string: String,
	}

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	enum TestEnum {
		NewTypeVariant(u8),
		StructVariant { a: u8, b: u8 },
		TupleVariant(u8, u8, u8),
		UnitVariant,
	}

	macro_rules! impl_test_x {
		($name:ident, $v:expr) => {
			#[test]
			fn $name() {
				test($v);
				test_be($v);
				test_undersized($v);
			}
		};
	}

	// Test Serde primitive types
	impl_test_x!(test_bool_true, true);
	impl_test_x!(test_bool_false, false);

	impl_test_x!(test_u8, 0x41 as u8);
	impl_test_x!(test_u16, 0x41 as u16);
	impl_test_x!(test_u32, 0x41 as u32);
	impl_test_x!(test_u64, 0x41 as u64);
	impl_test_x!(test_u128, 0x41 as u128);

	impl_test_x!(test_i8, 0x41 as i8);
	impl_test_x!(test_i16, 0x41 as i16);
	impl_test_x!(test_i32, 0x41 as i32);
	impl_test_x!(test_i64, 0x41 as i64);
	impl_test_x!(test_i128, 0x41 as i128);

	impl_test_x!(test_f32, 0x41 as f32);
	impl_test_x!(test_f64, 0x41 as f64);

	impl_test_x!(test_char, 'a');
	impl_test_x!(test_unicode_two_byte_char, 'ð');
	impl_test_x!(test_unicode_three_byte_char, 'ఈ');
	impl_test_x!(test_unicode_four_byte_char, '😶');

	#[test]
	fn test_bad_char() {
		assert!(Deserializer::from_bytes::<char>(&vec![0x80 as u8], false).is_err());
		assert!(Deserializer::from_bytes::<char>(&vec![0xC0 as u8, 0x00 as u8], false).is_err());
		assert!(Deserializer::from_bytes::<char>(&vec![], false).is_err());
	}

	// Test Serde String
	impl_test_x!(test_string, String::from("test"));

	// Test Serde Option
	impl_test_x!(test_none, None::<u64>);
	impl_test_x!(test_some, Some(0x41));

	// Test Serde Units
	#[test]
	fn test_unit() {
		let serialized = Serializer::to_bytes(&(), false).unwrap();
		let deserialized: () = Deserializer::from_bytes(&serialized, false).unwrap();
		assert_eq!(
			(),
			deserialized,
			"{:?} serialized to {:?} and deserialized to {:?}",
			(),
			serialized,
			deserialized
		);
	}

	#[test]
	fn test_unit_struct() {
		let value = Unit {};
		let serialized = Serializer::to_bytes(&value, false).unwrap();
		let deserialized: Unit = Deserializer::from_bytes(&serialized, false).unwrap();
		assert_eq!(
			value, deserialized,
			"{:?} serialized to {:?} and deserialized to {:?}",
			value, serialized, deserialized
		);
	}

	// Test Serde Variants
	impl_test_x!(test_unit_variant, TestEnum::UnitVariant);
	impl_test_x!(test_newtype_variant, TestEnum::NewTypeVariant(0x41));
	impl_test_x!(test_tuple_variant, TestEnum::TupleVariant(0x41, 0x42, 0x43));

	// Test Serde Structs
	impl_test_x!(
		test_struct,
		Test {
			byte: 0x41,
			string: String::from("test"),
		}
	);
	impl_test_x!(test_newtype_struct, NewType(0x41));
	impl_test_x!(test_tuple_struct, TupleStruct(0x41, 0x42, 0x43));
	impl_test_x!(
		test_struct_variant,
		TestEnum::StructVariant { a: 0x41, b: 0x42 }
	);

	// Test Serde sequences
	impl_test_x!(test_vec, vec![0x41, 0x42, 0x43]);
	impl_test_x!(test_byte_array, [0x41 as u8, 0x42 as u8, 0x43 as u8]);
	impl_test_x!(test_array, [0x41, 0x42, 0x43]);

	#[test]
	fn test_map() {
		let mut v: HashMap<String, char> = HashMap::new();
		v.insert(String::from("a"), 'a');
		v.insert(String::from("b"), 'b');
		test(v.clone());
		test_be(v.clone());
	}

	// Test Serde Tuple
	impl_test_x!(test_tuple, ('a', 16, 0x41 as u8));

	fn test<'a, T>(value: T)
	where
		T: Serialize + Deserialize<'a> + std::fmt::Debug + PartialEq,
	{
		let serialized = Serializer::to_bytes(&value, false).unwrap();
		let deserialized: T = Deserializer::from_bytes(&serialized, false).unwrap();
		assert_eq!(
			value, deserialized,
			"{:?} serialized to {:?} and deserialized to {:?}",
			value, serialized, deserialized
		);
	}

	fn test_be<'a, T>(value: T)
	where
		T: Serialize + Deserialize<'a> + std::fmt::Debug + PartialEq,
	{
		let serialized = Serializer::to_bytes(&value, true).unwrap();
		let deserialized: T = Deserializer::from_bytes(&serialized, true).unwrap();
		assert_eq!(
			value, deserialized,
			"{:?} serialized to {:?} and deserialized to {:?}",
			value, serialized, deserialized
		);
	}

	fn test_undersized<'a, T>(value: T)
	where
		T: Serialize + Deserialize<'a> + std::fmt::Debug + PartialEq,
	{
		let serialized = Serializer::to_bytes(&value, false).unwrap();
		let shrunk = serialized[0..serialized.len() - 1].to_vec();

		assert!(Deserializer::from_bytes::<T>(&shrunk, false).is_err());
	}
}
