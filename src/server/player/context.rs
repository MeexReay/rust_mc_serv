use std::{hash::Hash, net::{SocketAddr, TcpStream}, sync::{Arc, RwLock, RwLockWriteGuard}};

use rust_mc_proto::MinecraftConnection;
use uuid::Uuid;

use crate::server::{context::ServerContext, protocol::ConnectionState, ServerError};

use super::protocol::ProtocolHelper;


pub struct ClientContext {
    pub server: Arc<ServerContext>,
    pub addr: SocketAddr,
    conn: RwLock<MinecraftConnection<TcpStream>>,
    handshake: RwLock<Option<Handshake>>,
    client_info: RwLock<Option<ClientInfo>>,
    player_info: RwLock<Option<PlayerInfo>>,
    state: RwLock<ConnectionState>
}

impl PartialEq for ClientContext {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl Hash for ClientContext {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}

impl Eq for ClientContext {}

impl ClientContext {
    pub fn new(
        server: Arc<ServerContext>, 
        conn: MinecraftConnection<TcpStream>
    ) -> ClientContext {
        ClientContext {
            server,
            addr: conn.get_ref().peer_addr().unwrap(),
            conn: RwLock::new(conn),
            handshake: RwLock::new(None),
            client_info: RwLock::new(None),
            player_info: RwLock::new(None),
            state: RwLock::new(ConnectionState::Handshake)
        }
    }

    pub fn set_handshake(self: &Arc<Self>, handshake: Handshake) {
        *self.handshake.write().unwrap() = Some(handshake);
    }

    pub fn set_client_info(self: &Arc<Self>, client_info: ClientInfo) {
        *self.client_info.write().unwrap() = Some(client_info);
    }

    pub fn set_player_info(self: &Arc<Self>, player_info: PlayerInfo) {
        *self.player_info.write().unwrap() = Some(player_info);
    }

    pub fn set_state(self: &Arc<Self>, state: ConnectionState) -> Result<(), ServerError> {
        *self.state.write().unwrap() = state.clone();

        for handler in self.server.packet_handlers(
            |o| o.on_state_priority()
        ).iter() {
            handler.on_state(self.clone(), state.clone())?;
        }

        Ok(())
    }

    pub fn handshake(self: &Arc<Self>) -> Option<Handshake> {
        self.handshake.read().unwrap().clone()
    }

    pub fn client_info(self: &Arc<Self>) -> Option<ClientInfo> {
        self.client_info.read().unwrap().clone()
    }

    pub fn player_info(self: &Arc<Self>) -> Option<PlayerInfo> {
        self.player_info.read().unwrap().clone()
    }

    pub fn state(self: &Arc<Self>) -> ConnectionState {
        self.state.read().unwrap().clone()
    }

    pub fn conn(self: &Arc<Self>) -> RwLockWriteGuard<'_, MinecraftConnection<TcpStream>> {
        self.conn.write().unwrap()
    }

    pub fn protocol_helper(self: &Arc<Self>) -> ProtocolHelper {
        ProtocolHelper::new(self.clone())
    }
}

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
