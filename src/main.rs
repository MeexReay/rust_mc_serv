use std::{io::{Read, Write}, net::TcpListener, thread, time::Duration};

use rust_mc_proto::{DataReader, DataWriter, MinecraftConnection, Packet};

use data::ServerError;
use pohuy::Pohuy;

pub mod pohuy;
pub mod data;

// Сделать настройку хоста через конфиг
pub const HOST: &str = "127.0.0.1:25565"; 

fn main() {
	let Ok(server) = TcpListener::bind(HOST) else {
	 	println!("Не удалось забиндить сервер на {}", HOST); 
		return;
	};

	println!("Сервер запущен на {}", HOST); 

	while let Ok((stream, addr)) = server.accept() {
		thread::spawn(move || { 
			println!("Подключение: {}", addr);

			// Установка таймаутов на чтение и запись
			// По умолчанию пусть будет 5 секунд, надо будет сделать настройку через конфиг
			stream.set_read_timeout(Some(Duration::from_secs(5))).pohuy();
			stream.set_write_timeout(Some(Duration::from_secs(5))).pohuy();

			// Обработка подключения
			// Если ошибка -> похуй
			handle_connection(MinecraftConnection::new(&stream)).pohuy();

			println!("Отключение: {}", addr);
		});
	}
}

fn handle_connection(
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
						conn.write_packet(&Packet::build(0x00, |packet| {
							// Отправка статуса
							// В будущем это надо будет переделать чтобы это отправлялось через Listener'ы а не самим ядром сервера
							// Хотя можно сделать и дефолтное значение через конфиг
							packet.write_string(&format!(
								// Пример статуса с дебаг-инфой
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
									\"description\": {{
										\"text\": \"pv: {protocol_version}, sp: {server_port}\nsa: {server_address}\"
									}},
									\"favicon\": \"data:image/png;base64,<data>\",
									\"enforcesSecureChat\": false
								}}"
							))
						})?)?;
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
			conn.write_packet(&Packet::build(0x00, |packet| {
				packet.write_string("{\"text\": \"This server is in developement!!\", \"color\": \"red\", \"bold\": true}")
			})?)?;

			// TODO: Чтение Configuration (возможно с примешиванием Listener'ов)
			// TODO: Обработчик пакетов Play (тоже трейт), который уже будет дергать Listener'ы
		},
		_ => {
			return Err(ServerError::UnknownPacket(format!("Неизвестный NextState при рукопожатии"))); 
		} // Тип подключения не рукопожатный
	}

	Ok(())
}
