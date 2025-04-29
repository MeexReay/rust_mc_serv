
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

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
	let mut conn = MinecraftConnection::new(stream);
	println!("Подключение: {}", addr);
	loop {
		let Ok(mut packet) = conn.read_packet().await else {break;};
		let Ok(x) = packet.read_bytes(packet.len()).await else {
			println!("X"); break;
		};
		println!("{}", String::from_utf8_lossy(&x));
	}
	conn.close().await;
}