mod de;
mod error;
mod options;
mod ser;

pub use de::Deserializer;
pub use error::*;
pub use options::*;
pub use ser::Serializer;

#[cfg(test)]
mod tests {
	use lowlevel_types::ascii;

	use crate::{Options, serdebinaryadv};

	macro_rules! impl_test_txx {
		($name:ident, $ty:ty) => {
			#[test]
			fn $name() {
				let v: $ty = 0x41 as $ty;
				let b = serdebinaryadv::Serializer::to_bytes(&v, Options::default()).unwrap();
				assert_eq!(b, v.to_le_bytes());
				let d: $ty =
					serdebinaryadv::Deserializer::from_bytes(b, Options::default()).unwrap();
				assert_eq!(v, d);
			}
		};
	}

	impl_test_txx!(test_u8, u8);

	impl_test_txx!(test_u16, u16);

	impl_test_txx!(test_u32, u32);

	impl_test_txx!(test_u64, u64);

	impl_test_txx!(test_i8, i8);

	impl_test_txx!(test_i16, i16);

	impl_test_txx!(test_i32, i32);

	impl_test_txx!(test_i64, i64);

	impl_test_txx!(test_f32, f32);

	impl_test_txx!(test_f64, f64);

	#[test]
	fn test_ascii_char() {
		let v: ascii::Char = ascii::Char(0x41);
		let b = serdebinaryadv::Serializer::to_bytes(&v, Options::default()).unwrap();
		assert_eq!(b, vec![0x41]);
		let d: ascii::Char =
			serdebinaryadv::Deserializer::from_bytes(b, Options::default()).unwrap();
		assert_eq!(v, d);
	}

	#[test]
	fn test_fixedlengthstring() {
		let v: ascii::FixedLengthString<1> = ascii::FixedLengthString::from([0x41]);
		let b = serdebinaryadv::Serializer::to_bytes(&v, Options::default()).unwrap();
		assert_eq!(b, vec![0x41]);
		let d: ascii::FixedLengthString<1> =
			serdebinaryadv::Deserializer::from_bytes(b, Options::default()).unwrap();
		assert_eq!(v, d);
	}
}
