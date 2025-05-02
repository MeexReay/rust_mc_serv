use rust_mc_proto::{DataReader, DataWriter};

use super::ServerError;

pub mod text_component;

// Трейт для чтения NBT-совместимых приколов
pub trait ReadWriteNBT<T>: DataReader + DataWriter {
    fn read_nbt(&mut self) -> Result<T, ServerError>;
    fn write_nbt(&mut self, val: &T) -> Result<(), ServerError>;
}