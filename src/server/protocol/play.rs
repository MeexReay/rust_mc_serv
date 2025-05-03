use std::{collections::HashMap, sync::Arc};

use craftflow_nbt::DynNBT;
use log::debug;
use rust_mc_proto::{DataWriter, Packet};
use serde_json::{json, Value};

use crate::server::{
    data::ReadWriteNBT, player::context::ClientContext, ServerError
};

use super::id::{clientbound::{self, configuration::REGISTRY_DATA}, serverbound};

pub fn send_registry_data(
    client: Arc<ClientContext>,
) -> Result<(), ServerError> {
    let registry_data = include_str!("registry_data.json");
    let registry_data: Value = serde_json::from_str(registry_data).unwrap();
    let registry_data = registry_data.as_object().unwrap();

    for (registry_name, registry_data) in registry_data {
        let registry_data = registry_data.as_object().unwrap();

        let mut packet = Packet::empty(clientbound::configuration::REGISTRY_DATA);
        packet.write_string(registry_name)?;

        packet.write_usize_varint(registry_data.len())?;

        debug!("sending registry: {registry_name}");

        for (key, value) in registry_data {
            packet.write_string(key)?;
            packet.write_boolean(true)?;

            let mut data = Vec::new();
            craftflow_nbt::to_writer(&mut data, value).unwrap();

            debug!("- {key}");
            
            packet.write_bytes(&data)?;
        }
        
        client.write_packet(&packet)?;
    }

    Ok(())
}

pub fn handle_configuration_state(
    client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {

	let mut p = Packet::empty(clientbound::configuration::KNOWN_PACKS);
	p.write_varint(1)?;
	p.write_string("minecraft")?;
	p.write_string("core")?;
	p.write_string("1.21.5")?;
	client.write_packet(&p)?;
	client.read_packet(serverbound::configuration::KNOWN_PACKS)?;

    send_registry_data(client.clone())?;

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

    loop {}

    Ok(())
}
