use crate::{Endianness, Error, Result};

pub struct Raw {
	data: Vec<u8>,
	offset: usize,
}

impl Raw {
	pub fn new() -> Self {
		Self {
			data: vec![],
			offset: 0,
		}
	}

	pub fn from(input: &Vec<u8>) -> Self {
		Self {
			data: input.to_owned(),
			offset: 0,
		}
	}

	pub fn peek(&self) -> Result<u8> {
		if self.data.len() < 1 {
			return Err(Error::Eof);
		}
		Ok(self.data[self.offset + 1])
	}

	pub fn next(&mut self) -> Result<u8> {
		if self.data.len() < 1 {
			return Err(Error::Eof);
		}
		self.offset += 1;
		Ok(self.data[self.offset])
	}

	pub fn take(&mut self, n: usize) -> Result<Vec<u8>> {
		if self.data.len() < n {
			return Err(Error::Eof);
		}
		self.offset += n;
		Ok(self.data[0..n].to_vec())
	}

	pub fn take_u8(&mut self) -> Result<u8> {
		self.next()
	}

	pub fn take_u16(&mut self, endianness: &Endianness) -> Result<u16> {
		let bytes = self.take(2).unwrap();
		match endianness {
			Endianness::Native => Ok(u16::from_ne_bytes(bytes.try_into().unwrap())),
			Endianness::Little => Ok(u16::from_le_bytes(bytes.try_into().unwrap())),
			Endianness::Big => Ok(u16::from_be_bytes(bytes.try_into().unwrap())),
		}
	}

	pub fn take_u32(&mut self, endianness: &Endianness) -> Result<u32> {
		let bytes = self.take(4).unwrap();
		match endianness {
			Endianness::Native => Ok(u32::from_ne_bytes(bytes.try_into().unwrap())),
			Endianness::Little => Ok(u32::from_le_bytes(bytes.try_into().unwrap())),
			Endianness::Big => Ok(u32::from_be_bytes(bytes.try_into().unwrap())),
		}
	}

	pub fn take_u64(&mut self, endianness: &Endianness) -> Result<u64> {
		let bytes = self.take(8).unwrap();
		match endianness {
			Endianness::Native => Ok(u64::from_ne_bytes(bytes.try_into().unwrap())),
			Endianness::Little => Ok(u64::from_le_bytes(bytes.try_into().unwrap())),
			Endianness::Big => Ok(u64::from_be_bytes(bytes.try_into().unwrap())),
		}
	}

	pub fn take_u128(&mut self, endianness: &Endianness) -> Result<u128> {
		let bytes = self.take(16).unwrap();
		match endianness {
			Endianness::Native => Ok(u128::from_ne_bytes(bytes.try_into().unwrap())),
			Endianness::Little => Ok(u128::from_le_bytes(bytes.try_into().unwrap())),
			Endianness::Big => Ok(u128::from_be_bytes(bytes.try_into().unwrap())),
		}
	}

	pub fn take_usize(&mut self, endianness: &Endianness) -> Result<usize> {
		let bytes = self.take(size_of::<usize>()).unwrap();
		match endianness {
			Endianness::Native => Ok(usize::from_ne_bytes(bytes.try_into().unwrap())),
			Endianness::Little => Ok(usize::from_le_bytes(bytes.try_into().unwrap())),
			Endianness::Big => Ok(usize::from_be_bytes(bytes.try_into().unwrap())),
		}
	}
}
