use std::{net::{SocketAddr, TcpStream}, sync::{atomic::{AtomicI32, AtomicU16, Ordering}, Arc, RwLock, RwLockWriteGuard}};

use itertools::Itertools;
use rust_mc_proto::{MinecraftConnection, Packet};

use crate::{config::ServerConfig, data::ServerError};

pub struct ServerContext {
    pub config: Arc<ServerConfig>,
    listeners: Vec<Box<dyn Listener>>,
    handlers: Vec<Box<dyn PacketHandler>>
}

impl ServerContext {
    pub fn new(config: Arc<ServerConfig>) -> ServerContext {
        ServerContext {
            config,
            listeners: Vec::new(),
            handlers: Vec::new()
        }
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
    protocol_version: AtomicI32,
    server_address: RwLock<String>,
    server_port: AtomicU16,
}

impl ClientContext {
    pub fn new(
        server: Arc<ServerContext>, 
        conn: MinecraftConnection<TcpStream>
    ) -> ClientContext {
        ClientContext {
            server,
            addr: conn.get_ref().peer_addr().unwrap(),
            conn: RwLock::new(conn),
            protocol_version: AtomicI32::default(),
            server_address: RwLock::new(String::new()),
            server_port: AtomicU16::default()
        }
    }

    pub fn handshake(
        self: &Arc<Self>, 
        protocol_version: i32, 
        server_address: String, 
        server_port: u16
    ) -> () {
        self.protocol_version.store(protocol_version, Ordering::SeqCst);
        self.server_port.store(server_port, Ordering::SeqCst);
        *self.server_address.write().unwrap() = server_address;
    }

    pub fn protocol_version(self: &Arc<Self>) -> i32 {
        self.protocol_version.load(Ordering::SeqCst)
    }

    pub fn server_port(self: &Arc<Self>) -> u16 {
        self.server_port.load(Ordering::SeqCst)
    }

    pub fn server_address(self: &Arc<Self>) -> String {
        self.server_address.read().unwrap().clone()
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

