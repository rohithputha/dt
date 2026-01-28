mod training_coordinator;
mod worker;
mod accumulator;
mod gradient;
mod worker_proxy;
mod worker_pool;
use crate::training_coordinator::TrainingCoordinator;


fn main() {
   let mut tc = TrainingCoordinator::new();
   tc.run_cycles();
}
