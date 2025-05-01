use std::{env::args, io::{Read, Write}, net::TcpListener, path::PathBuf, sync::Arc, thread, time::Duration};

use config::ServerConfig;
use rust_mc_proto::{DataReader, DataWriter, MinecraftConnection, Packet};

use data::{ServerError, TextComponent};
use pohuy::Pohuy;

pub mod config;
pub mod data;
pub mod pohuy; 

fn main() {
	// Получение аргументов
	let exec = args().next().expect("Неизвестная система");
	let args = args().skip(1).collect::<Vec<String>>();

	if args.len() > 1 {
		println!("Использование: {exec} [путь до файла конфигурации]"); 
		return;
	}

	// Берем путь из аргумента либо по дефолту берем "./server.toml"
	let config_path = PathBuf::from(args.get(0).unwrap_or(&"server.toml".to_string()));

	// Чтение конфига, если ошибка - выводим
	let config = match ServerConfig::load_from_file(config_path) {
		Some(config) => config,
		None => {
			println!("Ошибка чтения конфигурации");
			return;
		},
	};

	// Делаем немутабельную потокобезопасную ссылку на конфиг
	// Впринципе можно и просто клонировать сам конфиг в каждый сука поток ебать того рот ебать блять
	// но мы этого делать не будем чтобы не было мемори лик лишнего
	let config = Arc::new(config);

	// Биндим сервер где надо
	let Ok(server) = TcpListener::bind(&config.host) else {
	 	println!("Не удалось забиндить сервер на {}", &config.host); 
		return;
	};

	println!("Сервер запущен на {}", &config.host); 

	while let Ok((stream, addr)) = server.accept() {
		let config = config.clone();

		thread::spawn(move || { 
			println!("Подключение: {}", addr);

			// Установка таймаутов на чтение и запись
			// По умолчанию пусть будет 5 секунд, надо будет сделать настройку через конфиг
			stream.set_read_timeout(Some(Duration::from_secs(config.timeout))).pohuy();
			stream.set_write_timeout(Some(Duration::from_secs(config.timeout))).pohuy();

			// Обработка подключения
			// Если ошибка -> выводим
			match handle_connection(config, MinecraftConnection::new(&stream)) {
				Ok(_) => {},
				Err(error) => {
					println!("Ошибка подключения: {error:?}");
				},
			};

			println!("Отключение: {}", addr);
		});
	}
}

fn handle_connection(
	_: Arc<ServerConfig>, // Конфиг сервера (возможно будет использоаться в будущем)
	mut conn: MinecraftConnection<impl Read + Write> // Подключение
) -> Result<(), ServerError> {
	// Чтение рукопожатия
	let mut packet = conn.read_packet()?;

	if packet.id() != 0x00 { 
		return Err(ServerError::UnknownPacket(format!("Неизвестный пакет рукопожатия"))); 
	} // Айди пакета не рукопожатное - выходим из функции

	let protocol_version = packet.read_varint()?; // Получаем версия протокола, может быть отрицательным если наш клиент дэбил
	let server_address = packet.read_string()?; // Получаем домен/адрес сервера к которому пытается подключиться клиент, например "play.example.com", а не айпи
	let server_port = packet.read_unsigned_short()?; // Все тоже самое что и с адресом сервера и все потому же и за тем же
	let next_state = packet.read_varint()?; // Тип подключения: 1 для получения статуса и пинга, 2 и 3 для обычного подключения

	match next_state {
		1 => { // Тип подключения - статус
			loop {
				// Чтение запроса
				let packet = conn.read_packet()?;

				match packet.id() {
					0x00 => { // Запрос статуса
						let mut packet = Packet::empty(0x00);

						// Отправка статуса
						// В будущем это надо будет переделать чтобы это отправлялось через Listener'ы а не самим ядром сервера
						// Хотя можно сделать и дефолтное значение через конфиг
						packet.write_string(&format!(
							// Пример статуса
							"{{
								\"version\": {{
									\"name\": \"1.21.5\",
									\"protocol\": {protocol_version}
								}},
								\"players\": {{
									\"max\": 100,
									\"online\": 5,
									\"sample\": [
										{{
											\"name\": \"thinkofdeath\",
											\"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"
										}}
									]
								}},
								\"description\": {},
								\"favicon\": \"data:image/png;base64,<data>\",
								\"enforcesSecureChat\": false
							}}",

							// В MOTD пихаем дебаг инфу
							TextComponent::builder()
								.text(format!("pv: {protocol_version}, sp: {server_port}\nsa: {server_address}"))
								.color("red".to_string())
								.bold(true)
								.italic(true)
								.underlined(true)
								.build()
								.to_string()?
						))?;

						conn.write_packet(&packet)?;
					},
					0x01 => { // Пинг
						conn.write_packet(&packet)?; 
						// Просто отправляем этот же пакет обратно
						// ID такой-же, содержание тоже, так почему бы и нет?
					},
					_ => { 
						return Err(ServerError::UnknownPacket(format!("Неизвестный пакет при чтении запросов статуса"))); 
					}
				}
			}
		},
		2 | 3 => { // Тип подключения - игра
			// Отключение игрока с сообщением
			// Заглушка так сказать
			let mut packet = Packet::empty(0x00);

			packet.write_string(&TextComponent::builder()
				.text(format!("This server is in developement!!"))
				.color("gold".to_string())
				.bold(true)
				.build()
				.to_string()?)?;

			conn.write_packet(&packet)?;

			// TODO: Чтение Configuration (возможно с примешиванием Listener'ов)
			// TODO: Обработчик пакетов Play (тоже трейт), который уже будет дергать Listener'ы
		},
		_ => {
			return Err(ServerError::UnknownPacket(format!("Неизвестный NextState при рукопожатии"))); 
		} // Тип подключения не рукопожатный
	}

	Ok(())
}
