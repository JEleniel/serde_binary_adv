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

				let buf2: &mut Vec<u8> = &mut Vec::new();
				Serializer::write_bytes(buf2, &$v, true).unwrap();
				let deserialized2: $ty =
					Deserializer::read_bytes(&mut buf2.as_slice(), true).unwrap();
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

	#[test]
	fn test_array() {
		let v: [u64; 3] = [0x41, 0x42, 0x43];
		let buf: &mut Vec<u8> = &mut Vec::new();
		Serializer::write_bytes(buf, &v, false).unwrap();
		let deserialized: [u64; 3] = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);

		let buf2: &mut Vec<u8> = &mut Vec::new();
		Serializer::write_bytes(buf2, &v, true).unwrap();
		let deserialized2: [u64; 3] = Deserializer::read_bytes(&mut buf2.as_slice(), true).unwrap();
		assert_eq!(v, deserialized2);
	}

	#[test]
	fn test_borrowed_bytes() {
		let bytes: &[u8] = &[0x41, 0x42, 0x43, 0x44];
		let mut buf = [0u8; 16];
		let mut written = &mut buf[..];
		Serializer::write_bytes(&mut written, &bytes, false).unwrap();
		let used = 16 - written.len();
		let mut read = &buf[..used];
		let deserialized: &[u8] = Deserializer::read_bytes(&mut read, false).unwrap();
		assert_eq!(bytes, deserialized);

		let mut buf2 = [0u8; 16];
		let mut written2 = &mut buf2[..];
		Serializer::write_bytes(&mut written2, &bytes, true).unwrap();
		let used2 = 16 - written2.len();
		let mut read2 = &buf2[..used2];
		let deserialized2: &[u8] = Deserializer::read_bytes(&mut read2, true).unwrap();
		assert_eq!(bytes, deserialized2);
	}

	#[test]
	fn test_serialize_byte_buf() {
		let bytes = vec![0x41, 0x42, 0x43, 0x44];
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &bytes, false).unwrap();
		let deserialized: Vec<u8> = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(bytes, deserialized);

		let mut buf2 = Vec::new();
		Serializer::write_bytes(&mut buf2, &bytes, true).unwrap();
		let deserialized2: Vec<u8> = Deserializer::read_bytes(&mut buf2.as_slice(), true).unwrap();
		assert_eq!(bytes, deserialized2);
	}

	#[test]
	fn test_deserialize_byte_buf() {
		let orig = vec![0x41, 0x42, 0x43, 0x44];
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &orig, false).unwrap();
		let deserialized: Vec<u8> = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(orig, deserialized);

		let mut buf2 = Vec::new();
		Serializer::write_bytes(&mut buf2, &orig, true).unwrap();
		let deserialized2: Vec<u8> = Deserializer::read_bytes(&mut buf2.as_slice(), true).unwrap();
		assert_eq!(orig, deserialized2);
	}

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

		let buf2: &mut Vec<u8> = &mut Vec::new();
		Serializer::write_bytes(buf2, &v, true).unwrap();
		let deserialized2: HashMap<String, char> =
			Deserializer::read_bytes(&mut buf2.as_slice(), true).unwrap();
		assert_eq!(v, deserialized2);
	}

	// Test Serde Tuple
	impl_test_x!(test_tuple, (char, i32, u8), ('a', 16, 0x41 as u8));

	#[test]
	fn test_empty_vec() {
		let v: Vec<u8> = vec![];
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: Vec<u8> = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);

		let mut buf2 = Vec::new();
		Serializer::write_bytes(&mut buf2, &v, true).unwrap();
		let deserialized2: Vec<u8> = Deserializer::read_bytes(&mut buf2.as_slice(), true).unwrap();
		assert_eq!(v, deserialized2);
	}

	#[test]
	fn test_empty_string() {
		let s = String::new();
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &s, false).unwrap();
		let deserialized: String = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(s, deserialized);

		let mut buf2 = Vec::new();
		Serializer::write_bytes(&mut buf2, &s, true).unwrap();
		let deserialized2: String = Deserializer::read_bytes(&mut buf2.as_slice(), true).unwrap();
		assert_eq!(s, deserialized2);
	}

	#[test]
	fn test_empty_map() {
		let v: HashMap<u8, u8> = HashMap::new();
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: HashMap<u8, u8> =
			Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);

		let mut buf2 = Vec::new();
		Serializer::write_bytes(&mut buf2, &v, true).unwrap();
		let deserialized2: HashMap<u8, u8> =
			Deserializer::read_bytes(&mut buf2.as_slice(), true).unwrap();
		assert_eq!(v, deserialized2);
	}

	#[test]
	fn test_unit_struct_roundtrip() {
		let v = Unit;
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: Unit = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_tuple_struct_roundtrip() {
		let v = TupleStruct(1, 2, 3);
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: TupleStruct =
			Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_struct_with_empty_string() {
		let v = Test {
			byte: 0,
			string: String::new(),
		};
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: Test = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_option_some_none() {
		let v: Option<u8> = Some(42);
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: Option<u8> =
			Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);

		let v: Option<u8> = None;
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: Option<u8> =
			Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_struct_variant_empty_fields() {
		#[derive(Serialize, Deserialize, Debug, PartialEq)]
		enum E {
			S {},
		}
		let v = E::S {};
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: E = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_tuple_variant_empty() {
		#[derive(Serialize, Deserialize, Debug, PartialEq)]
		enum E {
			T(),
		}
		let v = E::T();
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: E = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_enum_unit_variant() {
		#[derive(Serialize, Deserialize, Debug, PartialEq)]
		enum E {
			U,
		}
		let v = E::U;
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: E = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_enum_newtype_variant() {
		#[derive(Serialize, Deserialize, Debug, PartialEq)]
		enum E {
			N(u8),
		}
		let v = E::N(123);
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: E = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_enum_tuple_variant() {
		#[derive(Serialize, Deserialize, Debug, PartialEq)]
		enum E {
			T(u8, u8),
		}
		let v = E::T(1, 2);
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: E = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_enum_struct_variant() {
		#[derive(Serialize, Deserialize, Debug, PartialEq)]
		enum E {
			S { x: u8, y: u8 },
		}
		let v = E::S { x: 1, y: 2 };
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: E = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_map_non_string_key() {
		let mut v: HashMap<u8, u8> = HashMap::new();
		v.insert(1, 2);
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: HashMap<u8, u8> =
			Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}

	#[test]
	fn test_array_empty() {
		let v: [u8; 0] = [];
		let mut buf = Vec::new();
		Serializer::write_bytes(&mut buf, &v, false).unwrap();
		let deserialized: [u8; 0] = Deserializer::read_bytes(&mut buf.as_slice(), false).unwrap();
		assert_eq!(v, deserialized);
	}
}
