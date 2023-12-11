use std::sync::mpsc::{Receiver, Sender, SendError};
use std::thread;
use crate::minuterie::Heartbeat;

/// connects multiple receivers to the same sender
pub fn merge_channels(rxs: Vec<Receiver<Heartbeat>>, tx: Sender<Heartbeat>) {
    for rx in rxs {
        let tx = tx.clone();
        thread::spawn(move || {
            while let Ok(heartbeat) = rx.recv() {
                tx.send(heartbeat)?;
            }
            Ok::<(), SendError<Heartbeat>>(())
        });
    }
}

