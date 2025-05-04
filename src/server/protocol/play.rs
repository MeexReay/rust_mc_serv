use std::{io::Cursor, sync::Arc};

use rust_mc_proto::{read_packet, DataWriter, Packet};

use crate::server::{
    player::context::ClientContext, ServerError
};

use super::id::*;

pub fn send_update_tags(
    client: Arc<ClientContext>,
) -> Result<(), ServerError> {

    // rewrite this hardcode bullshit

    client.write_packet(&Packet::from_bytes(clientbound::configuration::UPDATE_TAGS, include_bytes!("update-tags.bin")))?;

    Ok(())
}

pub fn send_registry_data(
    client: Arc<ClientContext>,
) -> Result<(), ServerError> {

    // rewrite this hardcode bullshit

    let mut registry_data = Cursor::new(include_bytes!("registry-data.bin"));
    
    while let Ok(mut packet) = read_packet(&mut registry_data, None) {
        packet.set_id(clientbound::configuration::REGISTRY_DATA);
        client.write_packet(&packet)?;
    }

    Ok(())
}

pub fn process_known_packs(
    client: Arc<ClientContext>
) -> Result<(), ServerError> {
    let mut packet = Packet::empty(clientbound::configuration::KNOWN_PACKS);
	packet.write_varint(1)?;
	packet.write_string("minecraft")?;
	packet.write_string("core")?;
	packet.write_string("1.21.5")?;
	client.write_packet(&packet)?;

	client.read_packet(serverbound::configuration::KNOWN_PACKS)?;

    Ok(())
}

pub fn handle_configuration_state(
    client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {

    let mut packet = Packet::empty(clientbound::configuration::FEATURE_FLAGS);
	packet.write_varint(1)?;
	packet.write_string("minecraft:vanilla")?;
	client.write_packet(&packet)?;

	process_known_packs(client.clone())?;
    send_registry_data(client.clone())?;
    send_update_tags(client.clone())?;

    Ok(())
}

// Отдельная функция для работы с самой игрой
pub fn handle_play_state(
    client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {
    // Отключение игрока с сообщением
    // client.protocol_helper().disconnect(TextComponent::rainbow(
    //     "server is in developement suka".to_string(),
    // ))?;

	let mut packet = Packet::empty(clientbound::play::LOGIN);
	packet.write_int(0)?; // Entity ID
	packet.write_boolean(false)?; // Is hardcore
	packet.write_varint(4)?; // Dimension Names
	packet.write_string("minecraft:overworld")?;
	packet.write_string("minecraft:nether")?;
	packet.write_string("minecraft:the_end")?;
	packet.write_string("minecraft:overworld_caves")?;
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

    loop {} 
    
    // TODO: отдельный поток для чтения пакетов

    // TODO: переработка функции read_packet так чтобы когда 
    // делаешь read_any_packet, пакет отправлялся сначала всем другим 
    // функциям read_packet которые настроены на этот айди пакета,
    // а потом если таковых не осталось пакет возвращался

    Ok(())
}
