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

#[macro_export]
macro_rules! trigger_event {
    ($client:ident, $event:ident, $($arg:expr),* $(,)?) => {{
        paste::paste! {
            trigger_event!(@declare_mut_vars 0, $($arg),*);

            for handler in $client.server.listeners(
                |o| o.[<on_ $event _priority>]()
            ).iter() {
                handler.[<on_ $event>](
                    $client.clone(),
                    $(trigger_event!(@expand_arg 0, $arg)),*
                )?;
            }
        }
    }};

    (@declare_mut_vars $i:tt, &mut $head:expr, $($tail:tt)*) => {
        paste::paste! {
            let mut [<__arg $i>] = $head;
        }
        trigger_event!(@declare_mut_vars trigger_event!(@inc $i), $($tail)*);
    };
    (@declare_mut_vars $i:tt, $head:expr, $($tail:tt)*) => {
        trigger_event!(@declare_mut_vars trigger_event!(@inc $i), $($tail)*);
    };
    (@declare_mut_vars $_i:tt,) => {};

    (@expand_arg $i:tt, &mut $head:expr) => {
        paste::paste! { &mut [<__arg $i>] }
    };
    (@expand_arg $_i:tt, $head:expr) => {
        $head
    };

    (@inc 0) => { 1 };
    (@inc 1) => { 2 };
    (@inc 2) => { 3 };
    (@inc 3) => { 4 };
    (@inc 4) => { 5 };
    (@inc 5) => { 6 };
    (@inc 6) => { 7 };
    (@inc 7) => { 8 };
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