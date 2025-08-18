pub struct Options {
	pub endianness: Endianness,
	pub string_type: StringType,
}

pub enum Endianness {
	Native,
	Big,
	Little,
}

pub enum StringType {
	NullTerminated,
	SizeTagged,
	SizeTaggedNullTerminated,
}
