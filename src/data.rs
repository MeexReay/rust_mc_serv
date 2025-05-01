use std::{error::Error, fmt::Display};

use rust_mc_proto::ProtocolError;

// Ошибки сервера
#[derive(Debug)]
pub enum ServerError {
    UnknownPacket(String),
    Protocol(ProtocolError),
    ConnectionClosed
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

impl Error for ServerError {}

// Делаем чтобы ProtocolError мог переделываться в наш ServerError
impl From<ProtocolError> for ServerError {
    fn from(error: ProtocolError) -> ServerError {
        match error {
            // Если просто закрыто соединение, пеерделываем в нашу ошибку этого
            ProtocolError::ConnectionClosedError => {
                ServerError::ConnectionClosed
            },
            // Все остальное просто засовываем в обертку
            error => {
                ServerError::Protocol(error)
            },
        }
    }
}