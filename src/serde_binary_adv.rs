mod binaryerror;
mod common;
mod de;
mod ser;

pub use binaryerror::BinaryError;
pub use common::Result;
pub use de::Deserializer;
pub use ser::Serializer;

#[cfg(test)]
mod tests {
	use serde::{Deserialize, Serialize};

	use crate::{Deserializer, Serializer};

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	struct Test {
		pub byte: u8,
		pub string: String,
	}

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	enum TestEnum {
		First,
		Second,
	}

	macro_rules! impl_test_x {
		($name:ident, $v:expr) => {
			#[test]
			fn $name() {
				test($v);
				test_be($v);
			}
		};
	}

	impl_test_x!(test_bool_true, true);
	impl_test_x!(test_bool_false, false);

	impl_test_x!(test_u8, 0x41 as u8);
	impl_test_x!(test_u16, 0x41 as u16);
	impl_test_x!(test_u32, 0x41 as u32);
	impl_test_x!(test_u64, 0x41 as u64);

	impl_test_x!(test_i8, 0x41 as i8);
	impl_test_x!(test_i16, 0x41 as i16);
	impl_test_x!(test_i32, 0x41 as i32);
	impl_test_x!(test_i64, 0x41 as i64);

	impl_test_x!(test_f32, 0x41 as f32);
	impl_test_x!(test_f64, 0x41 as f64);

	impl_test_x!(test_char, 'a');
	impl_test_x!(test_unicode_two_byte_char, 'ð');
	impl_test_x!(test_unicode_three_byte_char, 'ఈ');
	impl_test_x!(test_unicode_four_byte_char, '😶');

	impl_test_x!(test_string, String::from("test"));

	impl_test_x!(test_tuple, ('a', 16, 0x41 as u8));

	impl_test_x!(test_array, [0x41, 0x42, 0x43]);

	impl_test_x!(
		test_struct,
		Test {
			byte: 0x41,
			string: String::from("test"),
		}
	);

	impl_test_x!(test_enum, TestEnum::Second);

	impl_test_x!(test_vec, vec![0x41, 0x42, 0x43]);

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
}
