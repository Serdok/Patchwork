use super::messenger::MessengerOperations;
//use super::packet::{KeepAlive, Packet};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::sleep;
use std::time;
use uuid::Uuid;

const KEEP_ALIVE_PERIOD: u64 = 15;
//const KEEP_ALIVE_VALUE: i64 = 16;

pub enum KeepAliveOperations {
    New(NewKeepAliveConnectionMessage),
}

#[derive(Debug)]
pub struct NewKeepAliveConnectionMessage {
    pub conn_id: Uuid,
}

pub fn start_keep_alive(
    receiver: Receiver<KeepAliveOperations>,
    _messenger: Sender<MessengerOperations>,
) {
    let mut conn_ids: Vec<Uuid> = Vec::new();

    loop {
        sleep(time::Duration::from_secs(KEEP_ALIVE_PERIOD));

        //after we wake up, add any new connections that were sent to us
        while let Ok(msg) = receiver.try_recv() {
            match msg {
                KeepAliveOperations::New(msg) => {
                    conn_ids.push(msg.conn_id);
                }
            }
        }

        //Turning this off for now- we don't need it for demo and it isn't properly implemented yet
        //for peers
        //
        //send all the keep alives
        //conn_ids.clone().into_iter().for_each(|conn_id| {
        //send_packet!(
        //messenger,
        //conn_id,
        //Packet::KeepAlive(KeepAlive {
        //id: KEEP_ALIVE_VALUE
        //})
        //)
        //.unwrap();
        //})
    }
}
