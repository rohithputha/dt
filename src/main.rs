mod training_coordinator;
mod worker;
mod accumulator;
mod gradient;
mod worker_proxy;
mod worker_pool;
mod protocol;
mod paramer_server;
mod param_server_proxy;
mod config_plane;

use crate::training_coordinator::TrainingCoordinator;

#[tokio::main]
async fn main() {
   let mut tc = TrainingCoordinator::new();
   tc.run_cycles().await;
}
