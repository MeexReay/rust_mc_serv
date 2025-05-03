pub mod id;
pub mod play;
pub mod handler;


#[derive(Debug, Clone)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play
}

