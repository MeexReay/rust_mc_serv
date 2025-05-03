use std::{collections::HashMap, sync::Arc};

use craftflow_nbt::DynNBT;
use rust_mc_proto::{DataWriter, Packet};
use serde_json::json;

use crate::server::{
    data::ReadWriteNBT, player::context::ClientContext, ServerError
};

use super::id::{clientbound::{self, configuration::REGISTRY_DATA}, serverbound};

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

	let mut data = Vec::new();
    craftflow_nbt::to_writer(&mut data, &json!(
        {
            "ambient_light": 0.0,
            "bed_works": 1,
            "coordinate_scale": 1.0,
            "effects": "minecraft:overworld",
            "has_ceiling": 0,
            "has_raids": 1,
            "has_skylight": 1,
            "height": 384,
            "infiniburn": "#minecraft:infiniburn_overworld",
            "logical_height": 384,
            "min_y": -64,
            "monster_spawn_block_light_limit": 0,
            "monster_spawn_light_level": {
                "max_inclusive": 7,
                "min_inclusive": 0,
                "type": "minecraft:uniform"
            },
            "natural": 1,
            "piglin_safe": 0,
            "respawn_anchor_works": 0,
            "ultrawarm": 0
        }
    )).unwrap();

	let mut p = Packet::empty(clientbound::configuration::REGISTRY_DATA);
	p.write_string("minecraft:dimension_type")?;
	p.write_varint(1)?;
	p.write_string("minecraft:overworld")?;
	p.write_boolean(true)?;
	// p.write_nbt(&DynNBT::Compound(HashMap::from_iter([
	// 	("bed_works".to_string(), DynNBT::Byte(1)),
	// 	("has_skylight".to_string(), DynNBT::Byte(1)),
	// 	("natural".to_string(), DynNBT::Byte(1)),
	// 	("coordinate_scale".to_string(), DynNBT::Double(1.0)),
	// 	("effects".to_string(), DynNBT::String("minecraft:overworld".to_string())),
	// ])))?;
	p.write_bytes(&data)?;
	client.write_packet(&p)?;

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

    Ok(())
}
