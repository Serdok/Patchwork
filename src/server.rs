use super::minecraft_protocol::read_var_int;

use super::packet::read;
use super::packet_router;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

use super::game_state::player::PlayerStateOperations;
use super::messenger::{MessengerOperations, NewConnectionMessage};
use std::sync::mpsc::Sender;

pub fn listen(messenger: Sender<MessengerOperations>, player_state: Sender<PlayerStateOperations>) {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let mut next_conn_id = 1;

    for stream in listener.incoming() {
        println!("connection");
        let stream = stream.unwrap();
        let messenger_clone = messenger.clone();
        let player_state_clone = player_state.clone();
        let conn_id = next_conn_id;
        thread::spawn(move || {
            handle_connection(stream, conn_id, messenger_clone, player_state_clone);
        });
        next_conn_id += 1;
    }
}

pub fn handle_connection(
    mut stream: TcpStream,
    conn_id: u64,
    messenger: Sender<MessengerOperations>,
    player_state: Sender<PlayerStateOperations>,
) {
    let mut state = 0;
    messenger
        .send(MessengerOperations::New(NewConnectionMessage {
            conn_id,
            socket: stream.try_clone().unwrap(),
        }))
        .unwrap();
    loop {
        match read_var_int(&mut stream) {
            Ok(length) => {
                let packet = read(&mut stream, state, length);
                packet_router::route_packet(
                    packet,
                    &mut state,
                    conn_id,
                    messenger.clone(),
                    player_state.clone(),
                );
            }
            Err(e) => {
                println!("conn closed due to {:?}", e);
                break;
            }
        }
    }
}