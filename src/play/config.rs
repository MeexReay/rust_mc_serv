use std::io::Cursor;
use std::sync::Arc;

use rust_mc_proto::{DataWriter, Packet, read_packet};

use crate::protocol::packet_id::*;
use crate::{ServerError, player::context::ClientContext};

pub fn send_update_tags(client: Arc<ClientContext>) -> Result<(), ServerError> {
	// TODO: rewrite this hardcode bullshit

	client.write_packet(&Packet::from_bytes(
		clientbound::configuration::UPDATE_TAGS,
		include_bytes!("update-tags.bin"),
	))
}

pub fn send_registry_data(client: Arc<ClientContext>) -> Result<(), ServerError> {
	// TODO: rewrite this hardcode bullshit

	let mut registry_data = Cursor::new(include_bytes!("registry-data.bin"));

	while let Ok(mut packet) = read_packet(&mut registry_data, None) {
		packet.set_id(clientbound::configuration::REGISTRY_DATA);
		client.write_packet(&packet)?;
	}

	Ok(())
}

// Добавки в Configuration стейт чтобы все работало
pub fn handle_configuration_state(
	client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {
	let mut packet = Packet::empty(clientbound::configuration::FEATURE_FLAGS);
	packet.write_varint(1)?;
	packet.write_string("minecraft:vanilla")?;
	client.write_packet(&packet)?;

	let mut packet = Packet::empty(clientbound::configuration::KNOWN_PACKS);
	packet.write_varint(1)?;
	packet.write_string("minecraft")?;
	packet.write_string("core")?;
	packet.write_string("1.21.5")?;
	client.write_packet(&packet)?;

	client.read_packet(&[serverbound::configuration::KNOWN_PACKS])?;

	send_registry_data(client.clone())?;
	send_update_tags(client.clone())
}
