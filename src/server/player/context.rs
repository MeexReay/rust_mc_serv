use std::{
    hash::Hash,
    net::{SocketAddr, TcpStream},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
};

use dashmap::DashMap;
use rust_mc_proto::{MinecraftConnection, Packet};
use uuid::Uuid;

use crate::server::{ServerError, context::ServerContext, protocol::ConnectionState};

use super::helper::ProtocolHelper;

// Клиент контекст
// Должен быть обернут в Arc для передачи между потоками
pub struct ClientContext {
    pub server: Arc<ServerContext>,
    pub addr: SocketAddr,
    conn: RwLock<MinecraftConnection<TcpStream>>,
    handshake: RwLock<Option<Handshake>>,
    client_info: RwLock<Option<ClientInfo>>,
    player_info: RwLock<Option<PlayerInfo>>,
    state: RwLock<ConnectionState>,
    packet_waiters: DashMap<usize, (u8, Sender<Packet>)>,
    read_loop: AtomicBool,
}

// Реализуем сравнение через адрес
// IPv4 не должен обманывать, иначе у нас случится коллапс
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
    pub fn new(server: Arc<ServerContext>, conn: MinecraftConnection<TcpStream>) -> ClientContext {
        ClientContext {
            server,
            addr: conn.get_ref().peer_addr().unwrap(),
            conn: RwLock::new(conn),
            handshake: RwLock::new(None),
            client_info: RwLock::new(None),
            player_info: RwLock::new(None),
            state: RwLock::new(ConnectionState::Handshake),
            packet_waiters: DashMap::new(),
            read_loop: AtomicBool::new(false),
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

        for handler in self
            .server
            .packet_handlers(|o| o.on_state_priority())
            .iter()
        {
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

    pub fn write_packet(self: &Arc<Self>, packet: &Packet) -> Result<(), ServerError> {
        let state = self.state();
        let mut packet = packet.clone();
        let mut cancelled = false;
        for handler in self
            .server
            .packet_handlers(|o| o.on_outcoming_packet_priority())
            .iter()
        {
            handler.on_outcoming_packet(
                self.clone(),
                &mut packet,
                &mut cancelled,
                state.clone(),
            )?;
            packet.get_mut().set_position(0);
        }
        if !cancelled {
            self.conn.write().unwrap().write_packet(&packet)?;
        }
        Ok(())
    }

    pub fn run_read_loop(
        self: &Arc<Self>,
        callback: impl Fn(Packet) -> Result<(), ServerError>,
    ) -> Result<(), ServerError> {
        let state = self.state();

        let mut conn = self.conn.read().unwrap().try_clone()?; // так можно делать т.к сокет это просто поинтер

        loop {
            let mut packet = conn.read_packet()?;
            let mut cancelled = false;
            for handler in self
                .server
                .packet_handlers(|o| o.on_incoming_packet_priority())
                .iter()
            {
                handler.on_incoming_packet(
                    self.clone(),
                    &mut packet,
                    &mut cancelled,
                    state.clone(),
                )?;
                packet.get_mut().set_position(0);
            }
            if !cancelled {
                let mut skip = false;
                for (_, (id, sender)) in self.packet_waiters.clone() {
                    if id == packet.id() {
                        sender.send(packet.clone()).unwrap();
                        skip = true;
                        break;
                    }
                }
                if !skip {
                    callback(packet.clone())?;
                }
            }
        }
    }

    pub fn read_any_packet(self: &Arc<Self>) -> Result<Packet, ServerError> {
        if self.read_loop.load(Ordering::SeqCst) {
            Err(ServerError::ReadLoopMode)
        } else {
            let state = self.state();

            let mut conn = self.conn.read().unwrap().try_clone()?; // так можно делать т.к сокет это просто поинтер

            loop {
                let mut packet = conn.read_packet()?;
                let mut cancelled = false;
                for handler in self
                    .server
                    .packet_handlers(|o| o.on_incoming_packet_priority())
                    .iter()
                {
                    handler.on_incoming_packet(
                        self.clone(),
                        &mut packet,
                        &mut cancelled,
                        state.clone(),
                    )?;
                    packet.get_mut().set_position(0);
                }
                if !cancelled {
                    break Ok(packet);
                }
            }
        }
    }

    pub fn read_packet(self: &Arc<Self>, id: u8) -> Result<Packet, ServerError> {
        if self.read_loop.load(Ordering::SeqCst) {
            let (tx, rx) = mpsc::channel::<Packet>();

            let key: usize = (&tx as *const Sender<Packet>).addr();
            self.packet_waiters.insert(key, (id, tx));

            loop {
                if let Ok(packet) = rx.recv() {
                    self.packet_waiters.remove(&key);
                    break Ok(packet);
                }
            }
        } else {
            let packet = self.read_any_packet()?;

            if packet.id() != id {
                Err(ServerError::UnexpectedPacket)
            } else {
                Ok(packet)
            }
        }
    }

    pub fn close(self: &Arc<Self>) {
        self.conn.write().unwrap().close();
    }
    pub fn set_compression(self: &Arc<Self>, threshold: Option<usize>) {
        self.conn.write().unwrap().set_compression(threshold);
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
    pub particle_status: i32,
}

#[derive(Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub uuid: Uuid,
}
