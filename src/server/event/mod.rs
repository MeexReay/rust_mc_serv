use rust_mc_proto::Packet;

use super::protocol::ConnectionState;

#[macro_export]
macro_rules! generate_handlers {
    ($name:ident $(, $arg_ty:ty)* $(,)?) => {
        paste::paste! {
            fn [<on_ $name _priority>](&self) -> i8 {
                0
            }

            fn [<on_ $name>](&self, _: std::sync::Arc<crate::server::player::context::ClientContext> $(, _: $arg_ty)*) -> Result<(), crate::server::ServerError> {
                Ok(())
            }
        }
    };
}

/// Отправляет пакет клиенту и проходит по пакет ханлдерам
/// Пример использования:
/// 
///     write_packet!(client, Handshake, packet);
/// 
/// `Handshake` это режим подключения (типы ConnectionState)
#[macro_export]
macro_rules! write_packet {
    ($client:expr, $state:ident, $packet:expr) => { 
        {
            let mut packet = $packet;
            let mut cancelled = false;
            for handler in $client.server.packet_handlers(
                |o| o.on_outcoming_packet_priority()
            ).iter() {
                handler.on_outcoming_packet($client.clone(), &mut packet, &mut cancelled, crate::server::protocol::ConnectionState::$state)?;
                packet.get_mut().set_position(0);
            }
            if !cancelled {
                $client.conn().write_packet(&packet)?;
            }
        }
    };
}

/// Читает пакет от клиента и проходит по пакет ханлдерам
/// Пример использования:
/// 
///     let packet = read_packet!(client, Handshake);
/// 
/// `Handshake` это режим подключения (типы ConnectionState)
#[macro_export]
macro_rules! read_packet {
    ($client:expr, $state:ident) => { 
        loop {
            let mut packet = $client.conn().read_packet()?;
            let mut cancelled = false;
            for handler in $client.server.packet_handlers(
                |o| o.on_incoming_packet_priority()
            ).iter() {
                handler.on_incoming_packet($client.clone(), &mut packet, &mut cancelled, crate::server::protocol::ConnectionState::$state)?;
                packet.get_mut().set_position(0);
            }
            if !cancelled {
                break packet;
            }
        }
    };
}

/// Пример использования:
/// 
///     trigger_event!(client, status, &mut response, state);
#[macro_export]
macro_rules! trigger_event {
    ($client:ident, $event:ident $(, $arg_ty:expr)* $(,)?) => {{
        paste::paste! {
            for handler in $client.server.listeners(
                |o| o.[<on_ $event _priority>]()
            ).iter() {
                handler.[<on_ $event>](
                    $client.clone()
                    $(, $arg_ty)*
                )?;
            }
        }
    }};
}

pub trait Listener: Sync + Send {
    generate_handlers!(status, &mut String);
    generate_handlers!(plugin_message, &mut String);
}

pub trait PacketHandler: Sync + Send {
    generate_handlers!(incoming_packet, &mut Packet, &mut bool, ConnectionState);
    generate_handlers!(outcoming_packet, &mut Packet, &mut bool, ConnectionState);
    generate_handlers!(state, ConnectionState);
}