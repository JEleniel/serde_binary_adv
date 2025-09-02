//! Versions of the Serde Binary Advanced Serializer and Deserializer optimized for
//! use with streams that implement Read/Write

mod de;
mod ser;

pub use de::Deserializer;
pub use ser::Serializer;

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::de::Deserializer;
	use super::ser::Serializer;

	use serde::{Deserialize, Serialize};

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
		($name:ident, $ty:ty, $v:expr) => {
			#[test]
			fn $name() {
				let buf: &mut Vec<u8> = &mut Vec::new();
				Serializer::write_bytes(buf, &$v, false).unwrap();
				let deserialized: $ty =
					Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
				assert_eq!($v as $ty, deserialized);

				Serializer::write_bytes(buf, &$v, true).unwrap();
				let deserialized2: $ty =
					Deserializer::read_bytes(&mut buf.as_slice(), true).unwrap();
				assert_eq!($v as $ty, deserialized2);
			}
		};
	}

	// Test Serde primitive types
	impl_test_x!(test_bool_true, bool, true);
	impl_test_x!(test_bool_false, bool, false);

	impl_test_x!(test_u8, u8, 0x41 as u8);
	impl_test_x!(test_u16, u16, 0x41 as u16);
	impl_test_x!(test_u32, u32, 0x41 as u32);
	impl_test_x!(test_u64, u64, 0x41 as u64);
	impl_test_x!(test_u128, u128, 0x41 as u128);

	impl_test_x!(test_i8, i8, 0x41 as i8);
	impl_test_x!(test_i16, i16, 0x41 as i16);
	impl_test_x!(test_i32, i32, 0x41 as i32);
	impl_test_x!(test_i64, i64, 0x41 as i64);
	impl_test_x!(test_i128, i128, 0x41 as i128);

	impl_test_x!(test_f32, f32, 0x41 as f32);
	impl_test_x!(test_f64, f64, 0x41 as f64);

	impl_test_x!(test_char, char, 'a');
	impl_test_x!(test_unicode_two_byte_char, char, 'ð');
	impl_test_x!(test_unicode_three_byte_char, char, 'ఈ');
	impl_test_x!(test_unicode_four_byte_char, char, '😶');

	#[test]
	fn test_bad_char() {
		assert!(Deserializer::read_bytes::<char>(&mut vec![0x80 as u8].as_slice(), false).is_err());
		assert!(
			Deserializer::read_bytes::<char>(&mut vec![0xC0 as u8, 0x00 as u8].as_slice(), false)
				.is_err()
		);
		assert!(Deserializer::read_bytes::<char>(&mut vec![].as_slice(), false).is_err());
	}

	// Test Serde String
	impl_test_x!(test_string, String, String::from("test"));

	// Test Serde Option
	impl_test_x!(test_none, Option<u64>, None::<u64>);
	impl_test_x!(test_some, Option<i32>, Some(0x41));

	// Test Serde Units
	impl_test_x!(test_unit, (), ());

	impl_test_x!(test_unit_struct, Unit, Unit {});

	// Test Serde Variants
	impl_test_x!(test_unit_variant, TestEnum, TestEnum::UnitVariant);
	impl_test_x!(
		test_newtype_variant,
		TestEnum,
		TestEnum::NewTypeVariant(0x41)
	);
	impl_test_x!(
		test_tuple_variant,
		TestEnum,
		TestEnum::TupleVariant(0x41, 0x42, 0x43)
	);

	// Test Serde Structs
	impl_test_x!(
		test_struct,
		Test,
		Test {
			byte: 0x41,
			string: String::from("test"),
		}
	);
	impl_test_x!(test_newtype_struct, NewType, NewType(0x41));
	impl_test_x!(
		test_tuple_struct,
		TupleStruct,
		TupleStruct(0x41, 0x42, 0x43)
	);
	impl_test_x!(
		test_struct_variant,
		TestEnum,
		TestEnum::StructVariant { a: 0x41, b: 0x42 }
	);

	// Test Serde sequences
	impl_test_x!(test_vec, Vec<u8>, vec![0x41 as u8, 0x42, 0x43]);
	impl_test_x!(
		test_byte_array,
		[u8; 3],
		[0x41 as u8, 0x42 as u8, 0x43 as u8]
	);
	impl_test_x!(test_array, [u64; 3], [0x41, 0x42, 0x43]);

	#[test]
	fn test_map() {
		let mut v: HashMap<String, char> = HashMap::new();
		v.insert(String::from("a"), 'a');
		v.insert(String::from("b"), 'b');
		let buf: &mut Vec<u8> = &mut Vec::new();
		Serializer::write_bytes(buf, &v, false).unwrap();
		let deserialized: HashMap<String, char> =
			Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);

		Serializer::write_bytes(buf, &v, true).unwrap();
		let deserialized2: HashMap<String, char> =
			Deserializer::read_bytes(&mut buf.as_slice(), true).unwrap();
		assert_eq!(v, deserialized2);
	}

	// Test Serde Tuple
	impl_test_x!(test_tuple, (char, i32, u8), ('a', 16, 0x41 as u8));
}
