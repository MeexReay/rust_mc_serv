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

// Пример использования:
//                                                         let packet_dst = trigger_packet!(packet_src, client, Handshake, incoming);             
//                                                                                             │           │        │          │                  
//                                                                        ┌────────────────────┼───────────┼────────┼──────────┘                  
//                                                                        │                    │           │        │
//                                                                        │              ┌─────┼───────────┘        │
//                                                                        │              │     │                    │
//                                                                        │              │     │                    └──────────────────┐
//                                                                        │              ▼     └───────────┐                           │
// Сделается вот такой вызов на всех packet_handler'ах:                   ▼                                ▼                           ▼
//                                                         handler.on_incoming_packet(client.clone(), &mut packet, ConnectionState::Handshake)   
// packet_src можно заменить на получение пакета, например: trigger_packet!(client.conn().read_packet()?, client, Handshake, incoming);
// В packet_dst будет лежать обратботанный пакет, прошедший через все хандлеры
// TODO: сделать чтобы можно было ваще отключить обработку
#[macro_export]
macro_rules! trigger_packet {
    ($packet:expr, $client:ident, $state:ident, $bound:ident) => { 
        {
            paste::paste! {
                let mut packet = $packet;
                for handler in $client.server.packet_handlers(
                    |o| o.[<on_ $bound _packet_priority>]()
                ).iter() {
                    handler.[<on_ $bound _packet>]($client.clone(), &mut packet, crate::server::protocol::ConnectionState::$state)?;
                }
                packet.get_mut().set_position(0);
                packet
            }
        }
    };
}

// Честно ни разу не проверял работу этого дерьма
// Пример использования:
// trigger_event!(client, status, $mut response, state);
// Сделается вот такой вызов на всех листенерах:
// listener.on_status(client.clone(), &mut response, state);
#[macro_export]
macro_rules! trigger_event {
    ($client:ident, $event:ident, $(, $arg_ty:ty)* $(,)?) => {{
        paste::paste! {
            for handler in $client.server.listeners(
                |o| o.[<on_ $event _priority>]()
            ).iter() {
                handler.[<on_ $event>](
                    $client.clone(),
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
    generate_handlers!(incoming_packet, &mut Packet, ConnectionState);
    generate_handlers!(outcoming_packet, &mut Packet, ConnectionState);
    generate_handlers!(state, ConnectionState);
}