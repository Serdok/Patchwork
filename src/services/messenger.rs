use super::map::Map;
use super::packet::{translate_outgoing, write, Packet};
use super::translation::TranslationInfo;
use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

pub trait Messenger {
    fn send_packet(&self, conn_id: Uuid, packet: Packet);
    fn broadcast_packet(&self, packet: Packet, source_conn_id: Option<Uuid>, local: bool);
    fn subscribe(&self, conn_id: Uuid, typ: SubscriberType);
    fn new_connection(&self, conn_id: Uuid, socket: TcpStream);
    fn update_translation(&self, conn_id: Uuid, map: Map);
}

impl Messenger for Sender<MessengerOperations> {
    fn send_packet(&self, conn_id: Uuid, packet: Packet) {
        self.send(MessengerOperations::Send(SendPacketMessage {
            conn_id,
            packet,
        }))
        .unwrap();
    }

    fn broadcast_packet(&self, packet: Packet, source_conn_id: Option<Uuid>, local: bool) {
        self.send(MessengerOperations::Broadcast(BroadcastPacketMessage {
            packet,
            source_conn_id,
            local,
        }))
        .unwrap();
    }

    fn subscribe(&self, conn_id: Uuid, typ: SubscriberType) {
        self.send(MessengerOperations::Subscribe(SubscribeMessage {
            conn_id,
            typ,
        }))
        .unwrap();
    }

    fn new_connection(&self, conn_id: Uuid, socket: TcpStream) {
        self.send(MessengerOperations::New(NewConnectionMessage {
            conn_id,
            socket,
        }))
        .unwrap();
    }

    fn update_translation(&self, conn_id: Uuid, map: Map) {
        self.send(MessengerOperations::UpdateTranslation(
            UpdateTranslationMessage { conn_id, map },
        ))
        .unwrap();
    }
}

pub enum MessengerOperations {
    Send(SendPacketMessage),
    Broadcast(BroadcastPacketMessage),
    Subscribe(SubscribeMessage),
    New(NewConnectionMessage),
    UpdateTranslation(UpdateTranslationMessage),
}

#[derive(Debug)]
pub struct SendPacketMessage {
    pub conn_id: Uuid,
    pub packet: Packet,
}

#[derive(Debug)]
pub struct UpdateTranslationMessage {
    pub conn_id: Uuid,
    pub map: Map,
}

#[derive(Debug)]
pub struct SubscribeMessage {
    pub conn_id: Uuid,
    pub typ: SubscriberType,
}

#[derive(Debug)]
pub enum SubscriberType {
    All,
    LocalOnly,
}

#[derive(Debug)]
pub struct BroadcastPacketMessage {
    pub packet: Packet,
    pub source_conn_id: Option<Uuid>,
    pub local: bool,
}

#[derive(Debug)]
pub struct NewConnectionMessage {
    pub conn_id: Uuid,
    pub socket: TcpStream,
}

pub fn start(receiver: Receiver<MessengerOperations>) {
    let mut connection_map = HashMap::<Uuid, TcpStream>::new();
    let mut local_only_broadcast_list = HashSet::<Uuid>::new();
    let mut all_broadcast_list = HashSet::<Uuid>::new();
    let mut translation_data = HashMap::<Uuid, TranslationInfo>::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            MessengerOperations::Send(msg) => {
                trace!(
                    "Sending packet {:?} to conn_id {:?}",
                    msg.packet.debug_print_type(),
                    msg.conn_id
                );
                if let Some(socket) = connection_map.get(&msg.conn_id) {
                    let mut socket_clone = socket.try_clone().unwrap();
                    let translated_packet = match translation_data.get(&msg.conn_id) {
                        Some(translation_data) => {
                            translate_outgoing(msg.packet, translation_data.clone())
                        }
                        None => msg.packet,
                    };
                    write(&mut socket_clone, translated_packet);
                    trace!("Send successful");
                } else {
                    trace!("Connection ID not found");
                }
            }
            MessengerOperations::Broadcast(msg) => {
                if msg.local {
                    trace!(
                        "Broadcasting packet {:?} from local source conn_id {:?}",
                        msg.packet.debug_print_type(),
                        msg.source_conn_id
                    );
                } else {
                    trace!(
                        "Broadcasting packet {:?} from remote source conn_id {:?}",
                        msg.packet.debug_print_type(),
                        msg.source_conn_id
                    );
                }
                (&all_broadcast_list).iter().for_each(|conn_id| {
                    if msg.source_conn_id.is_none() || msg.source_conn_id.unwrap() != *conn_id {
                        if let Some(socket) = connection_map.get(&conn_id) {
                            let mut socket_clone = socket.try_clone().unwrap();
                            let packet_clone = msg.packet.clone();
                            write(&mut socket_clone, packet_clone);
                        }
                    }
                });
                if msg.local {
                    (&local_only_broadcast_list).iter().for_each(|conn_id| {
                        if let Some(socket) = connection_map.get(&conn_id) {
                            let mut socket_clone = socket.try_clone().unwrap();
                            let packet_clone = msg.packet.clone();
                            write(&mut socket_clone, packet_clone);
                        }
                    });
                }
            }
            MessengerOperations::Subscribe(msg) => {
                trace!(
                    "Subscribing conn_id {:?} with type {:?}",
                    msg.conn_id,
                    msg.typ
                );
                match msg.typ {
                    SubscriberType::All => {
                        all_broadcast_list.insert(msg.conn_id);
                    }
                    SubscriberType::LocalOnly => {
                        local_only_broadcast_list.insert(msg.conn_id);
                    }
                }
            }
            MessengerOperations::New(msg) => {
                trace!(
                    "New Connection with conn_id {:?} on socket {:?}",
                    msg.conn_id,
                    msg.socket
                );
                connection_map.insert(msg.conn_id, msg.socket);
            }
            MessengerOperations::UpdateTranslation(msg) => {
                trace!(
                    "Updating connection map for conn_id {:?} to {:?}",
                    msg.conn_id,
                    msg.map
                );
                translation_data.insert(
                    msg.conn_id,
                    TranslationInfo {
                        state: 0,
                        map: msg.map,
                    },
                );
            }
        }
    }
}
