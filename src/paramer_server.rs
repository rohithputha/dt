use crate::protocol::Protocol;
use crate::param_server_proxy::ParamServerProxy;
use tokio::sync::broadcast::{Sender, Receiver};

pub struct ParamServer {
    id: u16,
    gradient_range: (u128, u128),
    tx: Sender<Protocol>,
    rx: Receiver<Protocol>,
    num_workers: u16,
}

impl ParamServer {
    pub fn new(id: u16, gradient_range: (u128, u128), tx: Sender<Protocol>, num_workers: u16) -> Self {
        let rx = tx.subscribe();
        ParamServer {
            id,
            gradient_range,
            tx,
            rx,
            num_workers,
        }
    }

    pub async fn listen(&mut self) {
        // spawn the param server proxy task
        let mut proxy = ParamServerProxy::new(self.id, self.tx.clone(), self.num_workers);
        tokio::spawn(async move {
            proxy.listen().await;
        });

        // tell the coordinator we are ready
        self.tx.send(Protocol::ToCoordinatorMessage {
            id: self.id,
            message: "wait".to_string(),
            w_type: "param_server".to_string(),
        }).unwrap();

        loop {
            match self.rx.recv().await {
                // forward compute / send down to our proxy
                Ok(Protocol::ToParamServerCommand { id, cmd }) if id == 0 || id == self.id => {
                    if cmd == "compute" || cmd == "send" {
                        self.tx.send(Protocol::ToParamServerProxyCommand {
                            id: self.id,
                            cmd,
                        }).unwrap();
                    }
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[ParamServer {}] Lagged by {} messages, re-syncing.", self.id, n);
                }
                Err(_) => break,
            }
        }
    }
}
