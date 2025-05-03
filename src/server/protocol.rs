use std::{io::Read, sync::Arc};

use super::{player::context::{ClientContext, ClientInfo, Handshake, PlayerInfo}, ServerError};
use rust_mc_proto::{DataReader, DataWriter, Packet};

use crate::{server::data::text_component::TextComponent, trigger_event};

// Тут будут все айди пакетов

pub const HANDSHAKE_ID: u8 = 0x00;
pub const STATUS_REQUEST_ID: u8 = 0x00;
pub const STATUS_RESPONSE_ID: u8 = 0x00;
pub const STATUS_PING_REQUEST_ID: u8 = 0x01;
pub const STATUS_PING_RESPONSE_ID: u8 = 0x01;
pub const LOGIN_START_ID: u8 = 0x00;
pub const LOGIN_SET_COMPRESSION_ID: u8 = 0x03;
pub const LOGIN_SUCCESS_ID: u8 = 0x02;
pub const LOGIN_ACKNOWLEDGED_ID: u8 = 0x03;
pub const CONFIGURATION_SERVERBOUND_PLUGIN_MESSAGE_ID: u8 = 0x02;
pub const CONFIGURATION_CLIENT_INFORMATION_ID: u8 = 0x00;
pub const CONFIGURATION_FINISH_ID: u8 = 0x03;
pub const CONFIGURATION_ACKNOWLEDGE_FINISH_ID: u8 = 0x03;


#[derive(Debug, Clone)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play
}

pub fn handle_connection(
	client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {
	// Чтение рукопожатия
	// Получение пакетов производится через client.conn(), 
	// ВАЖНО: не помещать сам client.conn() в переменные, 
	// он должен сразу убиваться иначе соединение гдето задедлочится
	let mut packet = client.read_packet(HANDSHAKE_ID)?;

	let protocol_version = packet.read_varint()?; // Получаем версия протокола, может быть отрицательным если наш клиент дэбил
	let server_address = packet.read_string()?; // Получаем домен/адрес сервера к которому пытается подключиться клиент, например "play.example.com", а не айпи
	let server_port = packet.read_unsigned_short()?; // Все тоже самое что и с адресом сервера и все потому же и за тем же
	let next_state = packet.read_varint()?; // Тип подключения: 1 для получения статуса и пинга, 2 и 3 для обычного подключения

	client.set_handshake(Handshake { protocol_version, server_address, server_port });

	match next_state {
		1 => { // Тип подключения - статус
			client.set_state(ConnectionState::Status)?; // Мы находимся в режиме Status

			loop {
				// Чтение запроса
				let mut packet = client.read_any_packet()?;

				match packet.id() {
					STATUS_REQUEST_ID => { // Запрос статуса
						let mut packet = Packet::empty(STATUS_RESPONSE_ID);

						// Дефолтный статус
						let mut status = "{
							\"version\": {
								\"name\": \"Error\",
								\"protocol\": 0
							},
							\"description\": {\"text\": \"Internal server error\"}
						}".to_string();

						// Опрос всех листенеров
						trigger_event!(client, status, &mut status);

						// Отправка статуса
						packet.write_string(&status)?;

						client.write_packet(&packet)?;
					},
					STATUS_PING_REQUEST_ID => { // Пинг
						// Раньше мы просто отправляли ему его-же пакет, но сейчас,
						// С приходом к власти констант айди-пакетов, нам приходится делать такое непотребство
						let timestamp = packet.read_long()?;
						let mut packet = Packet::empty(STATUS_PING_RESPONSE_ID);
						packet.write_long(timestamp)?;
						client.write_packet(&packet)?; 
					},
					_ => { 
						return Err(ServerError::UnexpectedPacket); 
					}
				}
			}
		},
		2 => { // Тип подключения - игра
			client.set_state(ConnectionState::Login)?; // Мы находимся в режиме Login

			// Читаем пакет Login Start
			let mut packet = client.read_packet(LOGIN_START_ID)?;

			let name = packet.read_string()?;
			let uuid = packet.read_uuid()?;

			client.set_player_info(PlayerInfo { name: name.clone(), uuid: uuid.clone() });

			if client.server.config.server.online_mode {
				// TODO: encryption packets
			}

			// Отправляем пакет Set Compression если сжатие указано
			if let Some(threshold) = client.server.config.server.compression_threshold {
				client.write_packet(&Packet::build(LOGIN_SET_COMPRESSION_ID, |p| p.write_usize_varint(threshold))?)?;
				client.set_compression(Some(threshold)); // Устанавливаем сжатие на соединении
			}

			// Отправка пакета Login Success
			client.write_packet(&Packet::build(LOGIN_SUCCESS_ID, |p| {
				p.write_uuid(&uuid)?;
				p.write_string(&name)?;
				p.write_varint(0)
			})?)?;

			client.read_packet(LOGIN_ACKNOWLEDGED_ID)?; // Пакет Login Acknowledged

			client.set_state(ConnectionState::Configuration)?; // Мы перешли в режим Configuration
			
			// Получение бренда клиента из Serverbound Plugin Message
			// Identifier канала откуда берется бренд: minecraft:brand
			let brand = loop {
				let mut packet = client.read_packet(CONFIGURATION_SERVERBOUND_PLUGIN_MESSAGE_ID)?; // Пакет Serverbound Plugin Message

				let identifier = packet.read_string()?;

				let mut data = Vec::new();
				packet.get_mut().read_to_end(&mut data).unwrap();

				if identifier == "minecraft:brand" { 
					break String::from_utf8_lossy(&data).to_string();
				} else {
					trigger_event!(client, plugin_message, &identifier, &data);
				}
			};

			let mut packet = client.read_packet(CONFIGURATION_CLIENT_INFORMATION_ID)?; // Пакет Client Information

			let locale = packet.read_string()?; // for example: en_us
			let view_distance = packet.read_signed_byte()?; // client-side render distance in chunks
			let chat_mode = packet.read_varint()?; // 0: enabled, 1: commands only, 2: hidden. See Chat#Client chat mode for more information. 
			let chat_colors = packet.read_boolean()?; // this settings does nothing on client but can be used on serverside
			let displayed_skin_parts = packet.read_byte()?; // bit mask https://minecraft.wiki/w/Java_Edition_protocol#Client_Information_(configuration)
			let main_hand = packet.read_varint()?; // 0 for left and 1 for right
			let enable_text_filtering = packet.read_boolean()?; // filtering text for profanity, always false for offline mode
			let allow_server_listings = packet.read_boolean()?; // allows showing player in server listings in status
			let particle_status = packet.read_varint()?; // 0 for all, 1 for decreased, 2 for minimal

			client.set_client_info(ClientInfo { 
				brand, 
				locale, 
				view_distance, 
				chat_mode, 
				chat_colors, 
				displayed_skin_parts, 
				main_hand, 
				enable_text_filtering, 
				allow_server_listings, 
				particle_status
			});

			// TODO: Заюзать Listener'ы чтобы они подмешивали сюда чото

			client.write_packet(&Packet::empty(CONFIGURATION_FINISH_ID))?;
			client.read_packet(CONFIGURATION_ACKNOWLEDGE_FINISH_ID)?;

			client.set_state(ConnectionState::Play)?; // Мы перешли в режим Play

			// Дальше работаем с режимом игры
			handle_play_state(client)?;
		},
		_ => { // Тип подключения не рукопожатный
			return Err(ServerError::UnexpectedPacket); 
		}
	}

	Ok(())
}

// Отдельная функция для работы с самой игрой
pub fn handle_play_state(
	client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> { 

	// Отключение игрока с сообщением
	client.protocol_helper().disconnect(TextComponent::rainbow("server is in developement suka".to_string()))?;

	// TODO: Сделать отправку пакетов Play

	Ok(())
}