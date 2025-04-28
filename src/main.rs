pub(crate) mod data;

use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use data::VarInt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// 1. Создаём TCP listener на порту 25565
	let listener = TcpListener::bind("127.0.0.1:25565").await?;
	println!("Listening on port 25565...");

	// 2. Асинхронно принимаем входящие соединения
	while let Ok((stream, _)) = listener.accept().await {
		// Для каждого соединения создаём отдельную задачу
		tokio::spawn(handle_connection(stream));
	}

	Ok(())
}

async fn handle_connection(mut stream: TcpStream) {
	let mut firstByte = [0];
	let Ok(n) = stream.read(&mut firstByte).await else { return };

	// let mut buffer = [0; 1024];

	// // 3. Читаем данные из потока
	// while let Ok(n) = stream.read(&mut buffer).await {
	// 	if n == 0 {
	// 		// Соединение закрыто
	// 		break;
	// 	}

	// 	// 4. Декодируем байты в UTF-8, пропуская ошибки
	// 	let received = String::from_utf8_lossy(&buffer[..n]);
	// 	print!("{}", received);
	// }

	// 5. Соединение автоматически закрывается при выходе из области видимости
}