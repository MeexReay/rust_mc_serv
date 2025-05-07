use std::sync::atomic::Ordering;
use std::{sync::Arc, thread, time::Duration};

use config::handle_configuration_state;
use helper::{
	send_game_event, send_keep_alive, send_system_message, set_center_chunk, sync_player_pos,
	unload_chunk,
};
use rust_mc_proto::{DataReader, DataWriter, Packet};
use uuid::Uuid;

use crate::event::Listener;
use crate::player::context::EntityInfo;
use crate::{
	ServerError, data::component::TextComponent, event::PacketHandler, player::context::ClientContext,
};

use crate::protocol::{ConnectionState, packet_id::*};

pub mod config;
pub mod helper;
pub mod planner;

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

pub struct PlayListener;

impl Listener for PlayListener {
	fn on_disconnect(&self, client: Arc<ClientContext>) -> Result<(), ServerError> {
		handle_disconnect(client)
	}
}

pub fn send_login(client: Arc<ClientContext>) -> Result<(), ServerError> {
	// Отправка пакета Login
	let mut packet = Packet::empty(clientbound::play::LOGIN);

	packet.write_int(client.entity_info().entity_id)?; // Entity ID
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
	packet.write_varint(0)?; // MOTION_BLOCKING - 0
	// bits per entry is ceil(log2(385)) = 9 where 385 is the world height
	// so, the length of the following array is (9 * 16 * 16) / 8 = 37
	// ... idk how it came to that
	packet.write_varint(37)?; // Length of the following long array 
	for _ in 0..37 {
		packet.write_long(0)?; // THIS WORKS ONLY BECAUSE OUR HEIGHT IS 0
	}

	// sending chunk data

	let mut chunk_data = Vec::new();

	// we want to fill the area from -64 to 0, so it will be 4 chunk sections

	for _ in 0..4 {
		chunk_data.write_short(4096)?; // non-air blocks count, 16 * 16 * 16 = 4096 stone blocks

		// blocks paletted container
		chunk_data.write_byte(0)?; // Bits Per Entry, use Single valued palette format
		chunk_data.write_varint(1)?; // block state id in the registry 

		// biomes palleted container
		chunk_data.write_byte(0)?; // Bits Per Entry, use Single valued palette format
		chunk_data.write_varint(1)?; // biome id in the registry
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

pub fn remove_player(
	receiver: Arc<ClientContext>,
	player: Arc<ClientContext>,
) -> Result<(), ServerError> {
	let mut packet = Packet::empty(clientbound::play::PLAYER_INFO_REMOVE);

	packet.write_varint(1)?;
	packet.write_uuid(&player.entity_info().uuid)?;

	receiver.write_packet(&packet)?;

	let mut packet = Packet::empty(clientbound::play::REMOVE_ENTITIES);

	packet.write_varint(1)?;
	packet.write_varint(player.entity_info().entity_id)?; // Entity ID

	receiver.write_packet(&packet)?;

	Ok(())
}

pub fn send_player(
	receiver: Arc<ClientContext>,
	player: Arc<ClientContext>,
) -> Result<(), ServerError> {
	let mut packet = Packet::empty(clientbound::play::PLAYER_INFO_UPDATE);

	packet.write_byte(0x01)?; // only Add Player 
	packet.write_varint(1)?; // players list
	packet.write_uuid(&player.entity_info().uuid)?; // player uuid
	packet.write_string(&player.player_info().unwrap().name)?; // player name
	packet.write_varint(0)?; // no properties

	receiver.write_packet(&packet)?;

	let mut packet = Packet::empty(clientbound::play::SPAWN_ENTITY);

	let (x, y, z) = player.entity_info().position();
	let (yaw, pitch) = player.entity_info().rotation();
	let (vel_x, vel_y, vel_z) = player.entity_info().velocity();

	packet.write_varint(player.entity_info().entity_id)?; // Entity ID
	packet.write_uuid(&player.entity_info().uuid)?; // Entity UUID
	packet.write_varint(148)?; // Entity type TODO: move to const
	packet.write_double(x)?;
	packet.write_double(y)?;
	packet.write_double(z)?;
	packet.write_signed_byte((pitch / 360.0 * 256.0) as i8)?;
	packet.write_signed_byte((yaw / 360.0 * 256.0) as i8)?;
	packet.write_signed_byte((yaw / 360.0 * 256.0) as i8)?; // head yaw TODO: make player head yaw field
	packet.write_varint(0)?;
	packet.write_short(vel_x as i16)?;
	packet.write_short(vel_y as i16)?;
	packet.write_short(vel_z as i16)?;

	receiver.write_packet(&packet)?;

	Ok(())
}

pub fn get_offline_uuid(name: &str) -> Uuid {
	let mut namespaces_bytes: [u8; 16] = [0; 16];
	for (i, byte) in format!("OfflinePlayer:{}", &name[..2])
		.as_bytes()
		.iter()
		.enumerate()
	{
		namespaces_bytes[i] = *byte;
	}
	let namespace = Uuid::from_bytes(namespaces_bytes);
	Uuid::new_v3(&namespace, (&name[2..]).as_bytes())
}

pub fn send_rainbow_message(
	client: &Arc<ClientContext>,
	message: String,
) -> Result<(), ServerError> {
	send_system_message(client.clone(), TextComponent::rainbow(message), false)
}

// Отдельная функция для работы с самой игрой
pub fn handle_play_state(
	client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {
	let player_name = client.player_info().unwrap().name;
	let player_uuid = get_offline_uuid(&client.player_info().unwrap().name); // TODO: authenticated uuid
	let entity_id = client
		.server
		.world
		.entity_id_counter
		.fetch_add(1, Ordering::SeqCst);

	client.set_entity_info(EntityInfo::new(entity_id, player_uuid));

	client.entity_info().set_position((8.0, 0.0, 8.0)); // set 8 0 8 as position

	thread::spawn({
		let client = client.clone();

		move || {
			let _ = client.run_read_loop();
			client.close();
		}
	});

	send_login(client.clone())?;
	sync_player_pos(client.clone(), 8.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)?;
	send_game_event(client.clone(), 13, 0.0)?; // 13 - Start waiting for level chunks
	// send_game_event(client.clone(), 3, 1.0)?; // 3 - Set gamemode, 1.0 - creative
	set_center_chunk(client.clone(), 0, 0)?;

	let mut chunks = Vec::new();

	let view_distance = client.client_info().unwrap().view_distance as i32 / 2;

	send_example_chunks_in_distance(client.clone(), &mut chunks, view_distance, (0, 0))?;

	// sync_player_pos(client.clone(), 8.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)?;

	// send_rainbow_message(&client, format!("Your IP: {}", client.addr))?;
	// send_rainbow_message(
	// 	&client,
	// 	format!("Your brand: {}", client.client_info().unwrap().brand),
	// )?;
	// send_rainbow_message(
	// 	&client,
	// 	format!("Your locale: {}", client.client_info().unwrap().locale),
	// )?;
	// send_rainbow_message(&client, format!("Your UUID: {}", client.entity_info().uuid))?;
	// send_rainbow_message(&client, format!("Your Name: {}", &player_name))?;
	// send_rainbow_message(&client, format!("Your Entity ID: {}", entity_id))?;

	for player in client.server.players() {
		if client.addr == player.addr {
			continue;
		}
		send_player(client.clone(), player.clone())?;
		send_player(player.clone(), client.clone())?;
	}

	thread::spawn({
		let client = client.clone();

		move || -> Result<(), ServerError> {
			while client.is_alive() {
				let mut packet = client.read_packet(&[
					serverbound::play::SET_PLAYER_POSITION,
					serverbound::play::SET_PLAYER_POSITION_AND_ROTATION,
					serverbound::play::SET_PLAYER_ROTATION,
					serverbound::play::CHAT_MESSAGE,
					serverbound::play::CLICK_CONTAINER,
					serverbound::play::CHAT_COMMAND,
					serverbound::play::SIGNED_CHAT_COMMAND,
				])?;

				match packet.id() {
					serverbound::play::CLICK_CONTAINER => {
						let window_id = packet.read_varint()?;
						let state_id = packet.read_varint()?;
						let slot = packet.read_short()?;
						let button = packet.read_byte()?;
						let mode = packet.read_varint()?;
						// i cannot read item slots now

						send_rainbow_message(&client, format!("index clicked: {slot}"))?;
					}
					serverbound::play::CHAT_COMMAND | serverbound::play::SIGNED_CHAT_COMMAND => {
						let command = packet.read_string()?;

						if command == "gamemode creative" {
							send_game_event(client.clone(), 3, 1.0)?; // 3 - Set gamemode
							send_rainbow_message(&client, format!("gamemode creative installed"))?;
						} else if command == "gamemode survival" {
							send_game_event(client.clone(), 3, 0.0)?; // 3 - Set gamemode
							send_rainbow_message(&client, format!("gamemode survival installed"))?;
						}
					}
					serverbound::play::CHAT_MESSAGE => {
						let message_text = packet.read_string()?;
						// skip remaining data coz they suck

						let mut message =
							TextComponent::rainbow(format!("{} said: ", client.player_info().unwrap().name));

						message.italic = Some(true);

						let text_message = TextComponent::builder()
							.color("white")
							.text(&message_text)
							.italic(false)
							.build();

						if let Some(extra) = &mut message.extra {
							extra.push(text_message);
						}

						for player in client.server.players() {
							send_system_message(player, message.clone(), false)?;
						}
					}
					serverbound::play::SET_PLAYER_POSITION => {
						let x = packet.read_double()?;
						let y = packet.read_double()?;
						let z = packet.read_double()?;
						let flags = packet.read_byte()?; // flags

						let prev = client.entity_info().position();

						for player in client.server.players() {
							if client.addr == player.addr {
								continue;
							}

							let mut packet = Packet::empty(clientbound::play::UPDATE_ENTITY_POSITION);
							packet.write_varint(client.entity_info().entity_id)?;
							packet.write_short((x * 4096.0 - prev.0 * 4096.0) as i16)?; // formula: currentX * 4096 - prevX * 4096
							packet.write_short((y * 4096.0 - prev.1 * 4096.0) as i16)?;
							packet.write_short((z * 4096.0 - prev.2 * 4096.0) as i16)?;
							packet.write_boolean(flags & 0x01 != 0)?;
							player.write_packet(&packet)?;
						}

						client.entity_info().set_position((x, y, z));
					}
					serverbound::play::SET_PLAYER_POSITION_AND_ROTATION => {
						let x = packet.read_double()?;
						let y = packet.read_double()?;
						let z = packet.read_double()?;
						let yaw = packet.read_float()?;
						let pitch = packet.read_float()?;
						let flags = packet.read_byte()?; // flags

						let prev = client.entity_info().position();

						for player in client.server.players() {
							if client.addr == player.addr {
								continue;
							}

							let mut packet =
								Packet::empty(clientbound::play::UPDATE_ENTITY_POSITION_AND_ROTATION);
							packet.write_varint(client.entity_info().entity_id)?;
							packet.write_short((x * 4096.0 - prev.0 * 4096.0) as i16)?; // formula: currentX * 4096 - prevX * 4096
							packet.write_short((y * 4096.0 - prev.1 * 4096.0) as i16)?;
							packet.write_short((z * 4096.0 - prev.2 * 4096.0) as i16)?;
							packet.write_signed_byte((yaw / 360.0 * 256.0) as i8)?;
							packet.write_signed_byte((pitch / 360.0 * 256.0) as i8)?;
							packet.write_boolean(flags & 0x01 != 0)?;
							player.write_packet(&packet)?;

							let mut packet = Packet::empty(clientbound::play::SET_HEAD_ROTATION);
							packet.write_varint(client.entity_info().entity_id)?;
							packet.write_signed_byte((yaw / 360.0 * 256.0) as i8)?;
							player.write_packet(&packet)?;
						}

						client.entity_info().set_position((x, y, z));
						client.entity_info().set_rotation((yaw, pitch));
					}
					serverbound::play::SET_PLAYER_ROTATION => {
						let yaw = packet.read_float()?;
						let pitch = packet.read_float()?;
						let flags = packet.read_byte()?; // flags

						for player in client.server.players() {
							if client.addr == player.addr {
								continue;
							}

							let mut packet = Packet::empty(clientbound::play::UPDATE_ENTITY_ROTATION);
							packet.write_varint(client.entity_info().entity_id)?;
							packet.write_signed_byte((yaw / 360.0 * 256.0) as i8)?;
							packet.write_signed_byte((pitch / 360.0 * 256.0) as i8)?;
							packet.write_boolean(flags & 0x01 != 0)?;
							player.write_packet(&packet)?;

							let mut packet = Packet::empty(clientbound::play::SET_HEAD_ROTATION);
							packet.write_varint(client.entity_info().entity_id)?;
							packet.write_signed_byte((yaw / 360.0 * 256.0) as i8)?;
							player.write_packet(&packet)?;
						}

						client.entity_info().set_rotation((yaw, pitch));
					}
					_ => {}
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
			let (x, _, z) = client.entity_info().position();

			let (chunk_x, chunk_z) = ((x / 16.0) as i64, (z / 16.0) as i64);
			let (chunk_x, chunk_z) = (chunk_x as i32, chunk_z as i32);

			set_center_chunk(client.clone(), chunk_x, chunk_z)?;
			send_example_chunks_in_distance(
				client.clone(),
				&mut chunks,
				view_distance,
				(chunk_x, chunk_z),
			)?;
		}

		// text animation
		{
			let animation_text = format!("Ticks alive: {}         жёпа", ticks_alive);
			let animation_index = ((ticks_alive + 40) % 300) as usize;
			let animation_end = animation_text.len() + 20;

			if animation_index < animation_end {
				let now_length = (animation_index + 1).min(animation_text.chars().count());
				let now_text = animation_text.chars().take(now_length).collect();

				send_system_message(client.clone(), TextComponent::rainbow(now_text), true)?;
			}
		}

		thread::sleep(Duration::from_millis(50)); // 1 tick
		ticks_alive += 1;
	}

	Ok(())
}

pub fn handle_disconnect(
	client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {
	for player in client.server.players() {
		if client.addr == player.addr {
			continue;
		}

		remove_player(player.clone(), client.clone())?;
	}

	Ok(())
}
