use std::time::Duration;

use crate::protocol::Protocol;
use crate::accumulator::Accumulator;
use crate::gradient::Gradient;
use tokio::sync::broadcast::{Sender, Receiver};

#[derive(Copy, Clone, PartialEq)]
enum State {
    Wait,
    Compute,
    Send,
}

pub struct ParamServerProxy {
    id: u16,
    tx: Sender<Protocol>,
    rx: Receiver<Protocol>,
    accumulator: Accumulator,
    state: State,
    num_workers: u16,
    avg_gradient: Option<Gradient>,
}

impl ParamServerProxy {
    pub fn new(id: u16, tx: Sender<Protocol>, num_workers: u16) -> Self {
        let rx = tx.subscribe();
        ParamServerProxy {
            id,
            tx,
            rx,
            accumulator: Accumulator::new(),
            state: State::Wait,
            num_workers,
            avg_gradient: None,
        }
    }

    pub async fn listen(&mut self) {
        loop {
            match self.rx.recv().await {
                // accumulate every gradient that hits the wire
                Ok(Protocol::GradientToParamServer { worker_proxy_id, gradient }) => {
                    self.accumulator.add_gradient(gradient);
                    println!("[ParamServerProxy {}] Accumulated gradient from worker {}, total {}/{}",
                             self.id, worker_proxy_id,
                             self.accumulator.total_collected, self.num_workers);

                    if self.accumulator.is_ready(self.num_workers as u32) {
                        // tell coordinator all shards for this param server are in
                        self.tx.send(Protocol::ToCoordinatorMessage {
                            id: self.id,
                            message: "receive_done".to_string(),
                            w_type: "param_server".to_string(),
                        }).unwrap();
                    }
                }

                Ok(Protocol::ToParamServerProxyCommand { id, cmd }) if id == self.id => {
                    match cmd.as_str() {
                        "compute" if self.state == State::Wait => {
                            self.state = State::Compute;
                            // simulate compute latency
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            self.avg_gradient = Some(self.accumulator.get_avg_gradient());
                            println!("[ParamServerProxy {}] Computed avg gradient: {:?}",
                                     self.id, self.avg_gradient.as_ref().unwrap().gr_vec);
                            self.tx.send(Protocol::ToCoordinatorMessage {
                                id: self.id,
                                message: "compute_done".to_string(),
                                w_type: "param_server".to_string(),
                            }).unwrap();
                        }
                        "send" if self.state == State::Compute => {
                            self.state = State::Send;
                            // broadcast the averaged gradient back to workers
                            if let Some(avg) = self.avg_gradient.take() {
                                self.tx.send(Protocol::GradientFromParamServer {
                                    param_server_proxy_id: self.id,
                                    gradient: avg,
                                }).unwrap();
                                println!("[ParamServerProxy {}] Sent avg gradient back to workers.", self.id);
                            }
                            // reset accumulator for the next cycle
                            self.accumulator.reset();
                            self.state = State::Wait;
                            self.tx.send(Protocol::ToCoordinatorMessage {
                                id: self.id,
                                message: "send_done".to_string(),
                                w_type: "param_server".to_string(),
                            }).unwrap();
                        }
                        _ => {}
                    }
                }

                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[ParamServerProxy {}] Lagged by {} messages, re-syncing.", self.id, n);
                }
                Err(_) => break,
            }
        }
    }
}
