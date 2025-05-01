mod data;
use data::{Packet, Server, Socket};

mod d;
use d::*;

use std::thread;

fn main() {
	let a = Buffer::new(vec![0x01,0xFF,0x33], 0);
	let b = a;
	let x = a.read(1);
	let x2 = a.read(1);

	// let Ok(server) = Server::new("127.0.0.1:25565") else {
	// 	println!("Не удалось забиндить сервер"); return;
	// };

	// loop {
	// 	let socket = server.accept();
	// 	thread::spawn(move || { handle_connection(socket); });
	// }
}

fn handle_connection(socket: Socket) {
	let Ok(packet) = Packet::read_from(&socket) else {return;};
	// пакет уже имеет свой размер (size) и данные (data)
	// надо поместить пакет в очередь, обработать по шаблону и отдать обработчику

	// fn on_keep_alive(socket: Socket, time: u64) {
	// 	if time != self.time {
	// 		socket.close()
	// 	}
	// }
}
