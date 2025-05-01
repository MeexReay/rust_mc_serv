use std::ops::Index;








pub enum BufferError {
	EndOfBuffer
}

pub struct Buffer {
	bytes: Vec<u8>,
	index: usize
}

impl Buffer {
	pub fn new(bytes: Vec<u8>, index: usize) -> Self {
		Buffer { bytes, index }
	}

	pub fn read(&self, size: usize) -> Result<Vec<u8>, BufferError> {
		if self.index + size >= self.bytes.len() {return Err(BufferError::EndOfBuffer);}
		// self.index += size;
		Ok(self.bytes[self.index..self.index+size-1].to_vec())
	}

	pub fn read2(&mut self, size: usize) -> Result<Vec<u8>, BufferError> {
		if self.index + size >= self.bytes.len() {return Err(BufferError::EndOfBuffer);}
		self.index += size;
		Ok(self.bytes[self.index..self.index+size-1].to_vec())
	}
}

pub trait Sas {
	fn ts(&mut self);
}

impl Sas for Buffer {
	fn ts(&mut self) {
		self.index += 1;
	}
}