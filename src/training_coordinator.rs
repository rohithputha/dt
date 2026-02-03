use crate::worker::Worker;
use crate::paramer_server::ParamServer;
use crate::config_plane::ConfigPlane;
use crate::protocol::Protocol;
use tokio::sync::broadcast::{Sender, Receiver};

#[derive(Debug)]
enum CoordinatorState {
    Initializing,
    WorkersComputing,
    WorkersSending,
    ParamServersReceiving,
    ParamServersComputing,
    ParamServersSending,
}

pub struct TrainingCoordinator {
    state: CoordinatorState,
    config_plane_tx: Sender<Protocol>,
    config_plane_rx: Receiver<Protocol>,

    num_workers: u16,
    num_param_servers: u16,
    max_cycles: u32,

    // per-message-type counters — always incremented on receipt, reset when consumed by a transition
    workers_waited: u16,
    workers_compute_done: u16,
    workers_send_done: u16,
    param_servers_waited: u16,
    param_servers_receive_done: u16,
    param_servers_compute_done: u16,
    param_servers_send_done: u16,

    cycle: u32,
}

impl TrainingCoordinator {
    pub fn new() -> Self {
        let config_plane = ConfigPlane::new();
        TrainingCoordinator {
            state: CoordinatorState::Initializing,
            config_plane_tx: config_plane.get_config_plane_tx(),
            config_plane_rx: config_plane.subscribe(),
            num_workers: 4,
            num_param_servers: 2,
            max_cycles: 3,
            workers_waited: 0,
            workers_compute_done: 0,
            workers_send_done: 0,
            param_servers_waited: 0,
            param_servers_receive_done: 0,
            param_servers_compute_done: 0,
            param_servers_send_done: 0,
            cycle: 0,
        }
    }

    pub async fn run_cycles(&mut self) {
        // spawn workers
        for i in 1..=self.num_workers {
            let tx = self.config_plane_tx.clone();
            tokio::spawn(async move {
                let mut worker = Worker::new(vec![0.0; 10], i, tx);
                worker.listen().await;
            });
        }

        // spawn param servers
        for i in 1..=self.num_param_servers {
            let tx = self.config_plane_tx.clone();
            let num_workers = self.num_workers;
            tokio::spawn(async move {
                let mut ps = ParamServer::new(i, (0, 9), tx, num_workers);
                ps.listen().await;
            });
        }

        // main coordinator message loop
        loop {
            match self.config_plane_rx.recv().await {
                Ok(Protocol::ToCoordinatorMessage { id, message, w_type }) => {
                    self.handle_message(id, &message, &w_type);
                    if self.cycle >= self.max_cycles {
                        println!("[Coordinator] Completed {} cycles. Exiting.", self.max_cycles);
                        break;
                    }
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[Coordinator] Lagged by {} messages, re-syncing.", n);
                }
                Err(_) => break,
            }
        }
    }

    fn send_to_workers(&self, cmd: &str) {
        self.config_plane_tx.send(Protocol::ToWorkerCommand {
            id: 0, // 0 = broadcast to all workers
            cmd: cmd.to_string(),
        }).unwrap();
    }

    fn send_to_param_servers(&self, cmd: &str) {
        self.config_plane_tx.send(Protocol::ToParamServerCommand {
            id: 0, // 0 = broadcast to all param servers
            cmd: cmd.to_string(),
        }).unwrap();
    }

    fn handle_message(&mut self, id: u16, message: &str, w_type: &str) {
        // always increment the matching counter
        match (w_type, message) {
            ("worker",       "wait")          => { self.workers_waited        += 1; }
            ("worker",       "compute_done")  => { self.workers_compute_done  += 1; }
            ("worker",       "send_done")     => { self.workers_send_done     += 1; }
            ("param_server", "wait")          => { self.param_servers_waited  += 1; }
            ("param_server", "receive_done")  => { self.param_servers_receive_done  += 1; }
            ("param_server", "compute_done")  => { self.param_servers_compute_done += 1; }
            ("param_server", "send_done")     => { self.param_servers_send_done    += 1; }
            _ => { return; }
        }

        println!("[Coordinator] [{:?}] {} {} (id {})", self.state, w_type, message, id);

        // cascading transition check — after one transition the new state's
        // condition may already be satisfied (counters arrived out of order)
        loop {
            let mut transitioned = false;
            match self.state {
                CoordinatorState::Initializing => {
                    if self.workers_waited >= self.num_workers
                        && self.param_servers_waited >= self.num_param_servers
                    {
                        println!("[Coordinator] All components ready. Starting cycle {}.", self.cycle);
                        self.workers_waited = 0;
                        self.param_servers_waited = 0;
                        self.send_to_workers("compute");
                        self.state = CoordinatorState::WorkersComputing;
                        transitioned = true;
                    }
                }
                CoordinatorState::WorkersComputing => {
                    if self.workers_compute_done >= self.num_workers {
                        println!("[Coordinator] All workers computed. Telling them to send.");
                        self.workers_compute_done = 0;
                        self.send_to_workers("send");
                        self.state = CoordinatorState::WorkersSending;
                        transitioned = true;
                    }
                }
                CoordinatorState::WorkersSending => {
                    if self.workers_send_done >= self.num_workers {
                        println!("[Coordinator] All workers sent. Waiting for param servers to accumulate.");
                        self.workers_send_done = 0;
                        self.state = CoordinatorState::ParamServersReceiving;
                        transitioned = true;
                    }
                }
                CoordinatorState::ParamServersReceiving => {
                    if self.param_servers_receive_done >= self.num_param_servers {
                        println!("[Coordinator] All param servers received gradients. Telling them to compute.");
                        self.param_servers_receive_done = 0;
                        self.send_to_param_servers("compute");
                        self.state = CoordinatorState::ParamServersComputing;
                        transitioned = true;
                    }
                }
                CoordinatorState::ParamServersComputing => {
                    if self.param_servers_compute_done >= self.num_param_servers {
                        println!("[Coordinator] All param servers computed avg. Telling them to send.");
                        self.param_servers_compute_done = 0;
                        self.send_to_param_servers("send");
                        self.state = CoordinatorState::ParamServersSending;
                        transitioned = true;
                    }
                }
                CoordinatorState::ParamServersSending => {
                    if self.param_servers_send_done >= self.num_param_servers {
                        self.param_servers_send_done = 0;
                        self.cycle += 1;
                        println!("[Coordinator] ===== Cycle {} complete =====\n", self.cycle);
                        if self.cycle < self.max_cycles {
                            self.send_to_workers("compute");
                            self.state = CoordinatorState::WorkersComputing;
                            transitioned = true;
                        }
                    }
                }
            }
            if !transitioned { break; }
        }
    }
}
