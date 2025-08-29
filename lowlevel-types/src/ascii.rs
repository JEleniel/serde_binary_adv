mod char;
mod error;
mod fixedlengthstring;

pub use char::Char;
pub use error::ASCIIError;
pub use fixedlengthstring::FixedLengthString;

#[cfg(test)]
mod tests {
	#[cfg(feature = "serde")]
	use serde_test::{Token, assert_de_tokens_error, assert_tokens};

	#[cfg(feature = "serde")]
	use crate::ascii::{self};

	#[test]
	#[cfg(feature = "serde")]
	fn test_char_ser_de() {
		let c = ascii::Char(0x41);

		assert_tokens(&c, &[Token::U8(0x41)]);

		assert_de_tokens_error::<ascii::Char>(
			&[Token::U16(0x0041)],
			"invalid type: integer `65`, expected a single ASCII character",
		);
	}

	#[test]
	#[cfg(feature = "serde")]
	fn test_fixedlengthstring_ser_de() {
		let s = ascii::FixedLengthString([ascii::Char(0x41); 1]);
		assert_tokens(&s, &[Token::BorrowedBytes(&[0x41 as u8])]);
		assert_de_tokens_error::<ascii::FixedLengthString<1>>(
			&[Token::BorrowedBytes(&[0x41; 2])],
			"invalid length 2, expected an array of 1 ASCII bytes",
		);
	}
}
