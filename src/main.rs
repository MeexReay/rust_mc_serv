
use std::net::SocketAddr;

use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use rust_mc_proto_tokio::{packet, prelude::*, MCConnTcp, MinecraftConnection, Packet, ProtocolError};


#[tokio::main]
async fn main() {
	let listener = match TcpListener::bind("127.0.0.1:25565").await {
		Ok(v) => v,
		Err(e) => { println!("Не удалось забиндить сервер: {}", e); return; }
	};

	while let Ok((stream, addr)) = listener.accept().await {
		tokio::spawn(handle_connection(stream, addr));
	}
}

async fn read_handshake_packet(mut packet: Packet) -> Result<(usize, String, u16, usize), ProtocolError> {
	if packet.id() != 0x00 { return Err(ProtocolError::ReadError)}
	Ok((
		packet.read_usize_varint().await?,
		packet.read_string().await?,
		packet.read_unsigned_short().await?,
		packet.read_usize_varint().await?
	))
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
	let mut conn = MinecraftConnection::new(stream);

	let Ok(packet) = conn.read_packet().await else {return;};
	let Ok((pv, host, port, ns)) = read_handshake_packet(packet).await else {return;};

	if ns == 2 {
		println!("\nПодключение: {}", addr);
		println!("Версия протокола: {pv}");
		println!("Хост: {host}");
		println!("Порт: {port}");
	}

	conn.close().await;
}
