use std::{io::Read, net::{SocketAddr, TcpListener, TcpStream}};



pub enum ServerError {
	ReadPacketError,
	ConnectionClosedError,
	ReadError,
	BindError,
	VarIntIsTooBig,
	PacketIsEnd
}



pub struct Packet {
	size: i32,
	data: Vec<u8>
}

impl Packet {
	pub fn read_from(socket: &Socket) -> Result<Self, ServerError> {
		let (size, n) = socket.read_varint_size()?;
		let data = socket.read((size - n as i32) as usize)?;
		Ok(Packet { size, data })
	}
}



pub struct Socket {
	stream: TcpStream,
	addr: SocketAddr
}

impl Socket {
	pub fn read(&self, size: usize) -> Result<Vec<u8>, ServerError>{
		let mut buf: Vec<u8> = vec![0; size];
		match (&self.stream).read(&mut buf) {
			Ok(n) => if n == size {
				Ok(buf)
			} else if n == 0 {
				Err(ServerError::ConnectionClosedError)
			} else {
				buf.truncate(n);
				buf.append(&mut self.read(size-n)?);
				Ok(buf)
			},
			Err(_) => Err(ServerError::ReadError)
		}
	}

	pub fn read_varint_size(&self) -> Result<(i32, u8), ServerError>{
		let mut result = 0i32;
		let mut offset = 0;
		let mut byte: u8;
		loop {
			byte = self.read(1)?[0];
			result |= ((byte & 0x7F) << offset) as i32;
			if (byte & 0x80) == 0 {break;};
			offset += 7;
			if offset >= 32 {return Err(ServerError::VarIntIsTooBig)}
		}
		Ok((result, offset / 7))
	}

	pub fn read_varint(&self) -> Result<i32, ServerError>{
		Ok(self.read_varint_size()?.0)
	}
}



pub struct Server {
	listener: TcpListener
}

impl Server {
	pub fn new(addr: &str) -> Result<Self, ServerError> {
		match TcpListener::bind(addr) {
			Ok(listener) => Ok(Server { listener }),
			Err(_) => Err(ServerError::BindError)
		}
	}

	pub fn accept(&self) -> Socket {
		loop {
			match self.listener.accept() {
				Ok((stream, addr)) => return Socket {stream, addr},
				Err(_) => continue
			}
		}
	}
}