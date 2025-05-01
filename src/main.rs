use std::{env::args, io::Read, net::TcpListener, path::PathBuf, sync::Arc, thread, time::Duration};

use config::Config;
use context::{ClientContext, Listener, PacketHandler, ServerContext};
use log::{debug, error, info};
use player::{ClientInfo, Handshake, PlayerInfo};
use rust_mc_proto::{DataReader, DataWriter, MinecraftConnection, Packet};

use data::{ServerError, TextComponent};
use pohuy::Pohuy;

pub mod config;
pub mod data;
pub mod context;
pub mod player;
pub mod pohuy; 


struct ExampleListener;

impl Listener for ExampleListener {
	fn on_status(&self, client: Arc<ClientContext>, response: &mut String) -> Result<(), ServerError> {
		*response = format!(
			"{{
				\"version\": {{
					\"name\": \"idk\",
					\"protocol\": {}
				}},
				\"players\": {{
					\"max\": 100,
					\"online\": 42,
					\"sample\": [
						{{
							\"name\": \"Жопа\",
							\"id\": \"00000000-0000-0000-0000-000000000000\"
						}}
					]
				}},
				\"description\": {},
				\"favicon\": \"data:image/png;base64,<data>\",
				\"enforcesSecureChat\": false
			}}",
			client.handshake().unwrap().protocol_version,
			TextComponent::builder()
				.text("Hello World! ")
				.extra(vec![
					TextComponent::builder()
						.text("Protocol: ")
						.color("gold")
						.extra(vec![
							TextComponent::builder()
								.text(&client.handshake().unwrap().protocol_version.to_string())
								.underlined(true)
								.build()
						])	
						.build(),
					TextComponent::builder()
						.text("\nServer Addr: ")
						.color("green")
						.extra(vec![
							TextComponent::builder()
								.text(&format!("{}:{}", 
									client.handshake().unwrap().server_address, 
									client.handshake().unwrap().server_port
								))
								.underlined(true)
								.build()
						])	
						.build()
				])
				.build()
				.as_json()?
		);

		Ok(())
	}
}

struct ExamplePacketHandler;

impl PacketHandler for ExamplePacketHandler {}


fn main() {
	colog::init();

	// Получение аргументов
	let exec = args().next().expect("Неизвестная система");
	let args = args().skip(1).collect::<Vec<String>>();

	if args.len() > 1 {
		info!("Использование: {exec} [путь до файла конфигурации]"); 
		return;
	}

	// Берем путь из аргумента либо по дефолту берем "./server.toml"
	let config_path = PathBuf::from(args.get(0).unwrap_or(&"server.toml".to_string()));

	// Чтение конфига, если ошибка - выводим
	let config = match Config::load_from_file(config_path) {
		Some(config) => config,
		None => {
			error!("Ошибка чтения конфигурации");
			return;
		},
	};

	// Делаем немутабельную потокобезопасную ссылку на конфиг
	// Впринципе можно и просто клонировать сам конфиг в каждый сука поток ебать того рот ебать блять
	// но мы этого делать не будем чтобы не было мемори лик лишнего
	let config = Arc::new(config);

	// Создаем контекст сервера
	// Передается во все подключения
	let mut server = ServerContext::new(config);

	server.add_listener(Box::new(ExampleListener)); // Добавляем пример листенера
	server.add_packet_handler(Box::new(ExamplePacketHandler)); // Добавляем пример пакет хандлера

	// Бетонируем сервер контекст от изменений
	let server = Arc::new(server);

	// Биндим сервер где надо
	let Ok(listener) = TcpListener::bind(&server.config.bind.host) else {
	 	error!("Не удалось забиндить сервер на {}", &server.config.bind.host); 
		return;
	};

	info!("Сервер запущен на {}", &server.config.bind.host); 

	while let Ok((stream, addr)) = listener.accept() {
		let server = server.clone();

		thread::spawn(move || { 
			info!("Подключение: {}", addr);

			// Установка таймаутов на чтение и запись
			// По умолчанию пусть будет 5 секунд, надо будет сделать настройку через конфиг
			stream.set_read_timeout(Some(Duration::from_secs(server.config.bind.timeout))).pohuy();
			stream.set_write_timeout(Some(Duration::from_secs(server.config.bind.timeout))).pohuy();

			// Оборачиваем стрим в майнкрафт конекшн лично для нашего удовольствия
			let conn = MinecraftConnection::new(stream);

			// Создаем контекст клиента
			// Передавется во все листенеры и хандлеры чтобы определять именно этот клиент
			let client = Arc::new(ClientContext::new(server, conn));

			// Обработка подключения
			// Если ошибка -> выводим
			match handle_connection(client) {
				Ok(_) => {},
				Err(error) => {
					error!("Ошибка подключения: {error:?}");
				},
			};

			info!("Отключение: {}", addr);
		});
	}
}

fn handle_connection(
	client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {
	// Чтение рукопожатия
	// Получение пакетов производится через client.conn(), 
	// ВАЖНО: не помещать сам client.conn() в переменные, 
	// он должен сразу убиваться иначе соединение гдето задедлочится
	let mut packet = client.conn().read_packet()?;

	if packet.id() != 0x00 { 
		return Err(ServerError::UnknownPacket(format!("Неизвестный пакет рукопожатия"))); 
	} // Айди пакета не рукопожатное - выходим из функции

	let protocol_version = packet.read_varint()?; // Получаем версия протокола, может быть отрицательным если наш клиент дэбил
	let server_address = packet.read_string()?; // Получаем домен/адрес сервера к которому пытается подключиться клиент, например "play.example.com", а не айпи
	let server_port = packet.read_unsigned_short()?; // Все тоже самое что и с адресом сервера и все потому же и за тем же
	let next_state = packet.read_varint()?; // Тип подключения: 1 для получения статуса и пинга, 2 и 3 для обычного подключения

	debug!("protocol_version: {protocol_version}");
	debug!("server_address: {server_address}");
	debug!("server_port: {server_port}");
	debug!("next_state: {next_state}");

	client.set_handshake(Handshake { protocol_version, server_address, server_port });

	match next_state {
		1 => { // Тип подключения - статус
			loop {
				// Чтение запроса
				let packet = client.conn().read_packet()?;

				match packet.id() {
					0x00 => { // Запрос статуса
						let mut packet = Packet::empty(0x00);

						// Дефолтный статус
						let mut status = "{
							\"version\": {
								\"name\": \"Error\",
								\"protocol\": 0
							},
							\"description\": {\"text\": \"Internal server error\"}
						}".to_string();

						// Опрос всех листенеров
						for listener in client.server.listeners( // Цикл по листенерам
							|o| o.on_status_priority() // Сортировка по приоритетности
						).iter() {
							listener.on_status(client.clone(), &mut status)?; // Вызов метода листенера
						}

						// Отправка статуса
						packet.write_string(&status)?;

						client.conn().write_packet(&packet)?;
					},
					0x01 => { // Пинг
						client.conn().write_packet(&packet)?; 
						// Просто отправляем этот же пакет обратно
						// ID такой-же, содержание тоже, так почему бы и нет?
					},
					_ => { 
						return Err(ServerError::UnknownPacket(format!("Неизвестный пакет при чтении запросов статуса"))); 
					}
				}
			}
		},
		2 => { // Тип подключения - игра
			// Мы находимся в режиме Login

			// Читаем пакет Login Start
			let mut packet = client.conn().read_packet()?;

			let name = packet.read_string()?;
			let uuid = packet.read_uuid()?;

			debug!("name: {name}");
			debug!("uuid: {uuid}");

			client.set_player_info(PlayerInfo { name: name.clone(), uuid: uuid.clone() });

			if client.server.config.server.online_mode {
				// TODO: encryption packets
			}

			// Отправляем пакет Set Compression если сжатие указано
			if let Some(threshold) = client.server.config.server.compression_threshold {
				client.conn().write_packet(&Packet::build(0x03, |p| p.write_usize_varint(threshold))?)?;
				client.conn().set_compression(Some(threshold)); // Устанавливаем сжатие на соединении
			}

			// Отправка пакета Login Success
			client.conn().write_packet(&Packet::build(0x02, |p| {
				p.write_uuid(&uuid)?;
				p.write_string(&name)?;
				p.write_varint(0)
			})?)?;

			let packet = client.conn().read_packet()?;

			if packet.id() != 0x03 {
				return Err(ServerError::UnknownPacket(format!("Неизвестный пакет при ожидании Login Acknowledged"))); 
			}

			// Мы перешли в режим Configuration
			
			// Получение бренда клиента из Serverbound Plugin Message
			// Identifier канала откуда берется бренд: minecraft:brand
			let brand = loop {
				let mut packet = client.conn().read_packet()?;

				if packet.id() == 0x02 { // Пакет Serverbound Plugin Message
					let identifier = packet.read_string()?;

					let mut data = Vec::new();
					packet.get_mut().read_to_end(&mut data).unwrap();

					if identifier == "minecraft:brand" { 
						break String::from_utf8_lossy(&data).to_string();
					} else {
						error!("unknown plugin message channel: {}", identifier);
					}
				} else {
					return Err(ServerError::UnknownPacket(format!("Неизвестный пакет при ожидании Serverbound Plugin Message"))); 
				};
			};

			debug!("brand: {brand}");

			let mut packet = client.conn().read_packet()?;

			// Пакет Client Information
			if packet.id() != 0x00 { 
				return Err(ServerError::UnknownPacket(format!("Неизвестный пакет при ожидании Client Information"))); 
			}

			let locale = packet.read_string()?; // for example: ru_RU
			let view_distance = packet.read_signed_byte()?; // client-side render distance in chunks
			let chat_mode = packet.read_varint()?; // 0: enabled, 1: commands only, 2: hidden. See Chat#Client chat mode for more information. 
			let chat_colors = packet.read_boolean()?; // this settings does nothing on client but can be used on serverside
			let displayed_skin_parts = packet.read_byte()?; // bit mask https://minecraft.wiki/w/Java_Edition_protocol#Client_Information_(configuration)
			let main_hand = packet.read_varint()?; // 0 for left and 1 for right
			let enable_text_filtering = packet.read_boolean()?; // filtering text for profanity, always false for offline mode
			let allow_server_listings = packet.read_boolean()?; // allows showing player in server listings in status
			let particle_status = packet.read_varint()?; // 0 for all, 1 for decreased, 2 for minimal

			debug!("locale: {locale}");
			debug!("view_distance: {view_distance}");
			debug!("chat_mode: {chat_mode}");
			debug!("chat_colors: {chat_colors}");
			debug!("displayed_skin_parts: {displayed_skin_parts}");
			debug!("main_hand: {main_hand}");
			debug!("enable_text_filtering: {enable_text_filtering}");
			debug!("allow_server_listings: {allow_server_listings}");
			debug!("particle_status: {particle_status}");

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

			client.conn().write_packet(&Packet::empty(0x03))?;

			let packet = client.conn().read_packet()?;

			if packet.id() != 0x03 {
				return Err(ServerError::UnknownPacket(format!("Неизвестный пакет при ожидании Acknowledge Finish Configuration"))); 
			}

			// Мы перешли в режим Play

			// Отключение игрока с сообщением
			// Отправляет в формате NBT TAG_String (https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/NBT#Specification:string_tag)
			client.conn().write_packet(&Packet::build(0x1C, |p| {
				let message = "server is in developmenet lol".to_string();
				p.write_byte(0x08)?; // NBT Type Name (TAG_String)
				p.write_unsigned_short(message.len() as u16)?; // String length in unsigned short
				p.write_bytes(message.as_bytes())
			})?)?;

			// TODO: Сделать отправку пакетов Play
		},
		_ => {
			return Err(ServerError::UnknownPacket(format!("Неизвестный NextState при рукопожатии"))); 
		} // Тип подключения не рукопожатный
	}

	Ok(())
}
