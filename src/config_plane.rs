
use tokio::sync::broadcast::{Sender, Receiver, channel};
use crate::protocol::Protocol;

pub struct ConfigPlane {
    tx: Sender<Protocol>,
}


impl ConfigPlane {
    pub fn new() -> Self {
        let (tx, mut rx) = channel::<Protocol>(1024);
        ConfigPlane {
            tx
        }
    }

    pub fn subscribe(&self) -> Receiver<Protocol> {
        self.tx.subscribe()
    }

}