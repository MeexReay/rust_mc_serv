use std::sync::Arc;

use rust_mc_proto::{DataWriter, Packet};

use crate::server::{
    ServerError, data::text_component::TextComponent, player::context::ClientContext,
};

use super::id::clientbound;

// Отдельная функция для работы с самой игрой
pub fn handle_play_state(
    client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {
    // Отключение игрока с сообщением
    // client.protocol_helper().disconnect(TextComponent::rainbow(
    //     "server is in developement suka".to_string(),
    // ))?;

	let mut packet = Packet::empty(clientbound::play::LOGIN);
	packet.write_int(10)?; // Entity ID
	packet.write_boolean(false)?; // Is hardcore
	packet.write_varint(1)?; // Dimension Names
	packet.write_string("minecraft:overworld")?;
	// packet.write_string("root/minecraft:nether")?;
	// packet.write_string("root/minecraft:the_end")?;
	packet.write_varint(0)?; // Max Players
	packet.write_varint(8)?; // View Distance
	packet.write_varint(5)?; // Simulation Distance
	packet.write_boolean(false)?; // Reduced Debug Info
	packet.write_boolean(true)?; // Enable respawn screen
	packet.write_boolean(false)?; // Do limited crafting

	packet.write_varint(0)?; // Dimension Type
	packet.write_string("minecraft:overworld")?; // Dimension Name
	packet.write_long(0x0f38f26ad09c3e20)?; // Hashed seed
	packet.write_byte(0)?; // Game mode
	packet.write_signed_byte(-1)?; // Previous Game mode
	packet.write_boolean(false)?; // Is Debug
	packet.write_boolean(true)?; // Is Flat
	packet.write_boolean(false)?; // Has death location
	packet.write_varint(20)?; // Portal cooldown
	packet.write_varint(60)?; // Sea level

	packet.write_boolean(false)?; // Enforces Secure Chat
	client.write_packet(&packet)?;

    Ok(())
}
