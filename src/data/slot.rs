use rust_mc_proto::{DataReader, DataWriter};

use crate::ServerError;

pub struct Slot {
	// TODO: write fields
}

pub trait ReadWriteSlot: DataReader + DataWriter {
	fn read_slot(&mut self) -> Result<Slot, ServerError>;
	fn write_slot(&mut self, val: Slot) -> Result<(), ServerError>;
}

pub struct HashedSlot {
	// TODO: write fields
}

pub trait ReadWriteHashedSlot: DataReader + DataWriter {
	fn read_hashed_slot(&mut self) -> Result<HashedSlot, ServerError>;
	fn write_hashed_slot(&mut self, val: HashedSlot) -> Result<(), ServerError>;
}

// TODO: implement traits for packet
