use num::traits::FromBytes;

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
}
