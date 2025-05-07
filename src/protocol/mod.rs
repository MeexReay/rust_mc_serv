pub mod handler;
pub mod packet_id;
pub mod play;

#[derive(Debug, Clone)]
pub enum ConnectionState {
	Handshake,
	Status,
	Login,
	Configuration,
	Play,
}
