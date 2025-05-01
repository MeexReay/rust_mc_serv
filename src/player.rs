use uuid::Uuid;

#[derive(Clone)]
pub struct Handshake {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
}

#[derive(Clone)]
pub struct ClientInfo {
    pub brand: String,
    pub locale: String,
    pub view_distance: i8,
    pub chat_mode: i32,
    pub chat_colors: bool,
    pub displayed_skin_parts: u8,
    pub main_hand: i32,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
    pub particle_status: i32
}

#[derive(Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub uuid: Uuid
}