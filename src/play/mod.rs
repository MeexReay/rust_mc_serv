use std::{sync::Arc, thread, time::Duration};

use config::handle_configuration_state;
use helper::{
	send_game_event, send_keep_alive, send_system_message, set_center_chunk, sync_player_pos,
	unload_chunk,
};
use rust_mc_proto::{DataReader, DataWriter, Packet};

use crate::{
	ServerError, data::text_component::TextComponent, event::PacketHandler,
	player::context::ClientContext,
};

use crate::protocol::{ConnectionState, packet_id::*};

pub mod config;
pub mod helper;

pub struct PlayHandler;

impl PacketHandler for PlayHandler {
	fn on_outcoming_packet(
		&self,
		client: Arc<ClientContext>,
		packet: &mut Packet,
		cancel: &mut bool,
		state: ConnectionState,
	) -> Result<(), ServerError> {
		if !*cancel	// проверяем что пакет не отмененный, облегчаем себе задачу, ведь проверять айди наверняка сложней
			&& state == ConnectionState::Configuration // проверяем стейт, т.к айди могут быть одинаковыми между стейтами
			&& packet.id() == clientbound::configuration::FINISH
		{
			handle_configuration_state(client)?; // делаем наши грязные дела
		}

		Ok(())
	}

	fn on_state(
		&self,
		client: Arc<ClientContext>,
		state: ConnectionState,
	) -> Result<(), ServerError> {
		if state == ConnectionState::Play {
			// перешли в режим плей, отлично! делаем дела

			handle_play_state(client)?;
		}

		Ok(())
	}
}

pub fn send_login(client: Arc<ClientContext>) -> Result<(), ServerError> {
	// Отправка пакета Login
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

	client.write_packet(&packet)
}

pub fn send_example_chunk(client: Arc<ClientContext>, x: i32, z: i32) -> Result<(), ServerError> {
	let mut packet = Packet::empty(clientbound::play::CHUNK_DATA_AND_UPDATE_LIGHT);

	packet.write_int(x)?;
	packet.write_int(z)?;

	// heightmap

	packet.write_varint(1)?; // heightmaps count
	packet.write_varint(0)?; // MOTION_BLOCKING
	packet.write_varint(256)?; // Length of the following long array (16 * 16 = 256)
	for _ in 0..256 {
		packet.write_long(0)?; // height - 0
	}

	// sending chunk data

	let mut chunk_data = Vec::new();

	// we want to fill the area from -64 to 0, so it will be 4 chunk sections

	for _ in 0..4 {
		chunk_data.write_short(4096)?; // non-air blocks count, 16 * 16 * 16 = 4096 stone blocks

		// blocks paletted container
		chunk_data.write_byte(0)?; // Bits Per Entry, use Single valued palette format
		chunk_data.write_varint(10)?; // block state id in the registry (1 for stone, 10 for dirt)

		// biomes palleted container
		chunk_data.write_byte(0)?; // Bits Per Entry, use Single valued palette format
		chunk_data.write_varint(27)?; // biome id in the registry
	}

	// air chunk sections

	for _ in 0..20 {
		chunk_data.write_short(0)?; // non-air blocks count, 0

		// blocks paletted container
		chunk_data.write_byte(0)?; // Bits Per Entry, use Single valued palette format
		chunk_data.write_varint(0)?; // block state id in the registry (0 for air)

		// biomes palleted container
		chunk_data.write_byte(0)?; // Bits Per Entry, use Single valued palette format
		chunk_data.write_varint(27)?; // biome id in the registry
	}

	packet.write_usize_varint(chunk_data.len())?;
	packet.write_bytes(&chunk_data)?;

	packet.write_byte(0)?;

	// light data

	packet.write_byte(0)?;
	packet.write_byte(0)?;
	packet.write_byte(0)?;
	packet.write_byte(0)?;
	packet.write_byte(0)?;
	packet.write_byte(0)?;

	client.write_packet(&packet)?;

	Ok(())
}

pub fn send_example_chunks_in_distance(
	client: Arc<ClientContext>,
	chunks: &mut Vec<(i32, i32)>,
	distance: i32,
	center: (i32, i32),
) -> Result<(), ServerError> {
	let mut new_chunks = Vec::new();

	for x in -distance + center.0..=distance + center.0 {
		for z in -distance + center.1..=distance + center.1 {
			if !chunks.contains(&(x, z)) {
				send_example_chunk(client.clone(), x as i32, z as i32)?;
			}
			new_chunks.push((x, z));
		}
	}

	for (x, z) in chunks.iter() {
		if !new_chunks.contains(&(*x, *z)) {
			unload_chunk(client.clone(), *x, *z)?;
		}
	}

	*chunks = new_chunks;

	Ok(())
}

// Отдельная функция для работы с самой игрой
pub fn handle_play_state(
	client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {
	thread::spawn({
		let client = client.clone();

		move || {
			let _ = client.run_read_loop();
			client.close();
		}
	});

	send_login(client.clone())?;
	sync_player_pos(client.clone(), 8.0, 3.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)?; // idk why, but now you need to set y to 3 here
	send_game_event(client.clone(), 13, 0.0)?; // 13 - Start waiting for level chunks
	set_center_chunk(client.clone(), 0, 0)?;

	let mut chunks = Vec::new();

	let view_distance = client.client_info().unwrap().view_distance as i32;

	send_example_chunks_in_distance(client.clone(), &mut chunks, view_distance, (0, 0))?;

	// sync_player_pos(client.clone(), 8.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)?;

	thread::spawn({
		let client = client.clone();

		move || -> Result<(), ServerError> {
			while client.is_alive() {
				let mut packet = client.read_any_packet()?;

				match packet.id() {
					serverbound::play::SET_PLAYER_POSITION => {
						let x = packet.read_double()?;
						let y = packet.read_double()?;
						let z = packet.read_double()?;
						let _ = packet.read_byte()?; // flags

						client.set_position((x, y, z));
					}
					serverbound::play::SET_PLAYER_POSITION_AND_ROTATION => {
						let x = packet.read_double()?;
						let y = packet.read_double()?;
						let z = packet.read_double()?;
						let yaw = packet.read_float()?;
						let pitch = packet.read_float()?;
						let _ = packet.read_byte()?; // flags

						client.set_position((x, y, z));
						client.set_rotation((yaw, pitch));
					}
					serverbound::play::SET_PLAYER_ROTATION => {
						let yaw = packet.read_float()?;
						let pitch = packet.read_float()?;
						let _ = packet.read_byte()?; // flags

						client.set_rotation((yaw, pitch));
					}
					_ => {
						client.push_packet_back(packet);
					}
				}
			}

			Ok(())
		}
	});

	let mut ticks_alive = 0u64;

	while client.is_alive() {
		if ticks_alive % 200 == 0 {
			// 10 secs timer
			send_keep_alive(client.clone())?;
		}

		if ticks_alive % 20 == 0 {
			// 1 sec timer
			let (x, _, z) = client.position();

			let (chunk_x, chunk_z) = ((x / 16.0) as i64, (z / 16.0) as i64);
			let (chunk_x, chunk_z) = (chunk_x as i32, chunk_z as i32);

			set_center_chunk(client.clone(), chunk_x, chunk_z)?;
			send_example_chunks_in_distance(
				client.clone(),
				&mut chunks,
				view_distance,
				(chunk_x, chunk_z),
			)?;

			// send_system_message(
			// 	client.clone(),
			// 	TextComponent::rainbow(format!("Pos: {} {} {}", x as i64, y as i64, z as i64)),
			// 	false,
			// )?;
		}

		send_system_message(
			client.clone(),
			TextComponent::rainbow(format!("Ticks alive: {}", ticks_alive)),
			true,
		)?;

		thread::sleep(Duration::from_millis(50)); // 1 tick
		ticks_alive += 1;
	}

	Ok(())
}
