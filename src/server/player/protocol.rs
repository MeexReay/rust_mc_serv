use std::{io::Read, sync::Arc};

use rust_mc_proto::{DataReader, DataWriter, Packet};

use crate::server::{data::text_component::TextComponent, data::ReadWriteNBT, protocol::ConnectionState, ServerError};

use super::context::ClientContext;


// Помощник в работе с протоколом
// Может быть использован где угодно, но сделан именно для листенеров и пакет хандлеров
// Через него удобно делать всякую одинаковую херь
// Возможно надо было бы сделать прям обязательный какойто структ через который только можно было отправлять пакеты ...
// ... но мне лень
// Пусть юзают подключение и отправляют пакеты через него если хотят
// Почему бы и нет если да
pub struct ProtocolHelper {
    client: Arc<ClientContext>,
    state: ConnectionState
}

impl ProtocolHelper {
    pub fn new(client: Arc<ClientContext>) -> Self {
        Self {
            state: client.state(),
            client
        }
    }
    
    pub fn disconnect(&self, reason: TextComponent) -> Result<(), ServerError> {
        let packet = match self.state {
            ConnectionState::Login => {
                let text = reason.as_json()?;
                Packet::build(0x00, |p| p.write_string(&text))?
            },
            ConnectionState::Configuration => {
                let mut packet = Packet::empty(0x02);
                packet.write_nbt(&reason)?;
                packet
            },
            ConnectionState::Play => {
                let mut packet = Packet::empty(0x1C);
                packet.write_nbt(&reason)?;
                packet
            },
            _ => {
                self.client.conn().close();
                return Ok(())
            },
        };
        self.client.conn().write_packet(&packet)?;
        Ok(())
    }

    /// Returns cookie content
    pub fn request_cookie(&self, id: &str) -> Result<Option<Vec<u8>>, ServerError> {
        match self.state {
            ConnectionState::Configuration => {
                let mut packet = Packet::empty(0x00);
                packet.write_string(id)?;
                self.client.conn().write_packet(&packet)?;

                let mut packet = self.client.conn().read_packet()?;
                packet.read_string()?;
                let data = if packet.read_boolean()? {
                    let n = packet.read_usize_varint()?;
                    Some(packet.read_bytes(n)?)
                } else {
                    None
                };

                Ok(data)
            },
            _ => Err(ServerError::UnexpectedState)
        }
    }

    /// Returns login plugin response - (message_id, payload)
    pub fn send_login_plugin_request(&self, id: i32, channel: &str, data: &[u8]) -> Result<(i32, Option<Vec<u8>>), ServerError> {
        match self.state {
            ConnectionState::Login => {
                let mut packet = Packet::empty(0x04);
                packet.write_varint(id)?;
                packet.write_string(channel)?;
                packet.write_bytes(data)?;
                self.client.conn().write_packet(&packet)?;

                let mut packet = self.client.conn().read_packet()?;
                let identifier = packet.read_varint()?;
                let data = if packet.read_boolean()? {
                    let mut data = Vec::new();
                    packet.get_mut().read_to_end(&mut data).unwrap();
                    Some(data)
                } else {
                    None
                };

                Ok((identifier, data))
            },
            _ => Err(ServerError::UnexpectedState)
        }
    }

    pub fn send_plugin_message(&self, channel: &str, data: &[u8]) -> Result<(), ServerError> {
        let mut packet = match self.state {
            ConnectionState::Configuration => Packet::empty(0x01),
            ConnectionState::Play => Packet::empty(0x18),
            _ => return Err(ServerError::UnexpectedState)
        };
        packet.write_string(channel)?;
        packet.write_bytes(data)?;
        self.client.conn().write_packet(&packet)?;
        Ok(())
    }
}