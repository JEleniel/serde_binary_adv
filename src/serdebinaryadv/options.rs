/// Options controlling the serialization and deserialization
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
	/// which endianness to use: native, big, or little
	pub endianness: Endianness,
	/// how to encode/decode strings, as null terminated, and with or without a size (in bytes) header
	pub string_type: StringType,
	/// how to encode/decode Unicode characters
	pub character_encoding: CharacterEncoding,
	/// whether all objects should be self describing
	pub self_describing: bool,
}

impl Default for Options {
	fn default() -> Self {
		Self {
			endianness: Endianness::Little,
			string_type: StringType::NullTerminated,
			character_encoding: CharacterEncoding::UTF8,
			self_describing: false,
		}
	}
}

/// which endiannedd to use for encode/decode
#[derive(Debug, PartialEq, Clone)]
pub enum Endianness {
	/// use the native endiannedd of the host
	Native,
	/// use big endian
	Big,
	/// use little endian
	Little,
}

/// how to encode/decode strings
#[derive(Debug, PartialEq, Clone)]
pub enum StringType {
	/// fixed length
	FixedLen,
	/// c type null terminated
	NullTerminated,
	/// prefixed with a length in bytes
	SizeTagged,
	/// prefixed with a length in bytes and null terminates
	SizeTaggedAndNullTerminated,
}

/// how to encode/decode characters
#[derive(Debug, PartialEq, Clone)]
pub enum CharacterEncoding {
	/// use ASCII encoding
	ASCII,
	/// use Unicode UTF-8 encoding
	UTF8,
	/// use Unicode UTF-16 encoding, follow the endianness setting
	UTF16,
}
