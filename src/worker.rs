use crate::worker_proxy::WorkerProxy;
use crate::protocol::Protocol;
use tokio::sync::broadcast::{Sender, Receiver};

pub struct Worker {
    id: u16,
    model_vec: Vec<f32>,
    config_plane_tx: Sender<Protocol>,
    config_plane_rx: Receiver<Protocol>,
}

impl Worker {
    pub fn new(model_vec: Vec<f32>, id: u16, tx: Sender<Protocol>) -> Self {
        let rx = tx.subscribe();
        Worker {
            id,
            model_vec,
            config_plane_tx: tx,
            config_plane_rx: rx,
        }
    }

    pub async fn listen(&mut self) {
        // spawn the worker proxy task
        let mut proxy = WorkerProxy::new(self.id, self.config_plane_tx.clone());
        tokio::spawn(async move {
            proxy.listen().await;
        });

        // tell the coordinator we are ready
        self.config_plane_tx.send(Protocol::ToCoordinatorMessage {
            id: self.id,
            message: "wait".to_string(),
            w_type: "worker".to_string(),
        }).unwrap();

        loop {
            match self.config_plane_rx.recv().await {
                // forward compute / send down to our proxy
                Ok(Protocol::ToWorkerCommand { id, cmd }) if id == 0 || id == self.id => {
                    if cmd == "compute" || cmd == "send" {
                        self.config_plane_tx.send(Protocol::ToWorkerProxyCommand {
                            id: self.id,
                            cmd,
                        }).unwrap();
                    }
                }

                // print averaged gradients that param servers broadcast back
                Ok(Protocol::GradientFromParamServer { param_server_proxy_id, gradient }) => {
                    println!("[Worker {}] Received avg gradient from PS {}: {:?}",
                             self.id, param_server_proxy_id, gradient.gr_vec);
                }

                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[Worker {}] Lagged by {} messages, re-syncing.", self.id, n);
                }
                Err(_) => break,
            }
        }
    }
}
