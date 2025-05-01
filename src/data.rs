use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ServerError {
	ReadPacketError,
	ConnectionClosedError,
	ReadError,
	BindError,
	VarIntIsTooBig,
	PacketIsEnd
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

impl Error for ServerError {}