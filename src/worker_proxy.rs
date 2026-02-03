use rand::Rng;

use std::time::Duration;

use crate::protocol::Protocol;
use crate::gradient::Gradient;
use tokio::sync::broadcast::{Sender, Receiver};

#[derive(Copy, Clone, PartialEq)]
enum State {
    Wait,
    Compute,
    Send,
}

pub struct WorkerProxy {
    id: u16,
    tx: Sender<Protocol>,
    rx: Receiver<Protocol>,
    state: State,
    gradient: Option<Gradient>,
}

impl WorkerProxy {
    pub fn new(id: u16, tx: Sender<Protocol>) -> Self {
        let rx = tx.subscribe();
        WorkerProxy {
            id,
            tx,
            rx,
            state: State::Wait,
            gradient: None,
        }
    }

    // generate a dummy 10-dimensional random gradient
    fn compute(&mut self) {
        let mut rng = rand::thread_rng();
        let gr_vec: Vec<f32> = (0..10).map(|_| rng.gen_range(-1.0_f32..1.0_f32)).collect();
        println!("[WorkerProxy {}] Computed dummy gradient: {:?}", self.id, gr_vec);
        self.gradient = Some(Gradient::new(gr_vec, self.id as u32, 0));
    }

    // broadcast the computed gradient so every ParamServerProxy picks it up
    fn send(&mut self) {
        if let Some(grad) = self.gradient.take() {
            self.tx.send(Protocol::GradientToParamServer {
                worker_proxy_id: self.id,
                gradient: grad,
            }).unwrap();
            println!("[WorkerProxy {}] Sent gradient to param server proxies.", self.id);
        }
    }

    pub async fn listen(&mut self) {
        loop {
            match self.rx.recv().await {
                Ok(Protocol::ToWorkerProxyCommand { id, cmd }) if id == self.id => {
                    match cmd.as_str() {
                        // turns out this is the shorter way to match the incoming command and check the state
                        "compute" if self.state == State::Wait => {
                            self.state = State::Compute;
                            self.compute();
                            // simulate compute latency
                            tokio::time::sleep(Duration::from_millis(150)).await;
                            // notify coordinator
                            self.tx.send(Protocol::ToCoordinatorMessage {
                                id: self.id,
                                message: "compute_done".to_string(),
                                w_type: "worker".to_string(),
                            }).unwrap();
                        }
                        "send" if self.state == State::Compute => {
                            self.state = State::Send;
                            self.send();
                            // notify coordinator
                            self.tx.send(Protocol::ToCoordinatorMessage {
                                id: self.id,
                                message: "send_done".to_string(),
                                w_type: "worker".to_string(),
                            }).unwrap();
                            self.state = State::Wait;
                        }
                        _ => {}
                    }
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[WorkerProxy {}] Lagged by {} messages, re-syncing.", self.id, n);
                }
                Err(_) => break,
            }
        }
    }
}
