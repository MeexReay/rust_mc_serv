use std::{hash::Hash, net::{SocketAddr, TcpStream}, sync::{Arc, RwLock, RwLockWriteGuard}};

use dashmap::DashMap;
use itertools::Itertools;
use rust_mc_proto::{MinecraftConnection, Packet};
use uuid::Uuid;

use crate::{config::Config, data::ServerError, player::{ClientInfo, Handshake, PlayerInfo}};

pub struct ServerContext {
    pub config: Arc<Config>,
    pub clients: DashMap<SocketAddr, Arc<ClientContext>>,
    listeners: Vec<Box<dyn Listener>>,
    handlers: Vec<Box<dyn PacketHandler>>
}

impl ServerContext {
    pub fn new(config: Arc<Config>) -> ServerContext {
        ServerContext {
            config,
            listeners: Vec::new(),
            handlers: Vec::new(),
            clients: DashMap::new()
        }
    }

    pub fn get_player_by_uuid(self: &Arc<Self>, uuid: Uuid) -> Option<Arc<ClientContext>> {
        self.clients.iter()
            .find(|o| {
                let info = o.player_info();
                if let Some(info) = info {
                    info.uuid == uuid
                } else {
                    false
                }
            })
            .map(|o| o.clone())
    }

    pub fn get_player_by_name(self: &Arc<Self>, name: &str) -> Option<Arc<ClientContext>> {
        self.clients.iter()
            .find(|o| {
                let info = o.player_info();
                if let Some(info) = info {
                    info.name == name
                } else {
                    false
                }
            })
            .map(|o| o.clone())
    }

    pub fn players(self: &Arc<Self>) -> Vec<Arc<ClientContext>> {
        self.clients.iter()
            .filter(|o| o.player_info().is_some())
            .map(|o| o.clone())
            .collect()
    }

    pub fn add_packet_handler(&mut self, handler: Box<dyn PacketHandler>) {
        self.handlers.push(handler);
    }

    pub fn add_listener(&mut self, listener: Box<dyn Listener>) {
        self.listeners.push(listener);
    }

    pub fn packet_handlers<F, K>(
        self: &Arc<Self>, 
        sort_by: F
    ) -> Vec<&Box<dyn PacketHandler>>
    where 
        K: Ord,
        F: FnMut(&&Box<dyn PacketHandler>) -> K 
    {
        self.handlers.iter().sorted_by_key(sort_by).collect_vec()
    }

    pub fn listeners<F, K>(
        self: &Arc<Self>, 
        sort_by: F
    ) -> Vec<&Box<dyn Listener>>
    where 
        K: Ord,
        F: FnMut(&&Box<dyn Listener>) -> K 
    {
        self.listeners.iter().sorted_by_key(sort_by).collect_vec()
    }
}

pub struct ClientContext {
    pub server: Arc<ServerContext>,
    pub conn: RwLock<MinecraftConnection<TcpStream>>,
    pub addr: SocketAddr,
    pub handshake: RwLock<Option<Handshake>>,
    pub client_info: RwLock<Option<ClientInfo>>,
    pub player_info: RwLock<Option<PlayerInfo>>
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
            player_info: RwLock::new(None)
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

    pub fn handshake(self: &Arc<Self>) -> Option<Handshake> {
        self.handshake.read().unwrap().clone()
    }

    pub fn client_info(self: &Arc<Self>) -> Option<ClientInfo> {
        self.client_info.read().unwrap().clone()
    }

    pub fn player_info(self: &Arc<Self>) -> Option<PlayerInfo> {
        self.player_info.read().unwrap().clone()
    }

    pub fn conn(self: &Arc<Self>) -> RwLockWriteGuard<'_, MinecraftConnection<TcpStream>> {
        self.conn.write().unwrap()
    }
}

pub trait Listener: Sync + Send {
    fn on_status_priority(&self) -> i8 { 0 }
    fn on_status(&self, _: Arc<ClientContext>, _: &mut String) -> Result<(), ServerError> { Ok(()) }
}

pub trait PacketHandler: Sync + Send {
    fn on_incoming_packet_priority(&self) -> i8 { 0 }
    fn on_incoming_packet(&self, _: Arc<ClientContext>, _: &mut Packet) -> Result<(), ServerError> { Ok(()) }

    fn on_outcoming_packet_priority(&self) -> i8 { 0 }
    fn on_outcoming_packet(&self, _: Arc<ClientContext>, _: &mut Packet) -> Result<(), ServerError> { Ok(()) }
}

pub struct Player {

}
