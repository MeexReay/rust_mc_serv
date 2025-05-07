pub mod handler;
pub mod packet_id;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
	Handshake,
	Status,
	Login,
	Configuration,
	Play,
}
