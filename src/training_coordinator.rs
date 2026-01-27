 
use crate::worker::Worker;
use crate::accumulator::Accumulator;
use crate::gradient::Gradient;
use std::fmt;
use std::thread;
use std::sync::{Arc, Mutex};
enum TrainingState {
    Collecting,
    Aggregating,
    Updating, 
    Broadcasting,
}

pub struct TrainingCoordinator {
    state: TrainingState,
    workers: Vec<Arc<Mutex<Worker>>>,
    accumulator: Arc<Mutex<Accumulator>>,
    avg_grad: Option<Gradient>,
}

impl fmt::Display for TrainingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str = match self {
            TrainingState::Collecting => "Collecting",
            TrainingState::Aggregating => "Aggregating",
            TrainingState::Updating => "Updating",
            TrainingState::Broadcasting => "Broadcasting",
        };
        write!(f, "{}", state_str)
    }
}

impl TrainingCoordinator {
    pub fn new() -> Self {
        let workers =vec![
            Arc::new(Mutex::new(Worker::new(vec![0.0; 10],1))),
            Arc::new(Mutex::new(Worker::new(vec![0.0; 10],2))),
            Arc::new(Mutex::new(Worker::new(vec![0.0; 10],3))),
            Arc::new(Mutex::new(Worker::new(vec![0.0; 10],4))),
        ];


        let accumulator = Accumulator::new();
        TrainingCoordinator {
            state: TrainingState::Collecting,
            workers,
            accumulator: Arc::new(Mutex::new(accumulator)),
            avg_grad: None,
        }
    }

    fn transition_state(&mut self){
        self.state = match self.state {
            TrainingState::Collecting => TrainingState::Aggregating,
            TrainingState::Aggregating => TrainingState::Updating,
            TrainingState::Updating => TrainingState::Broadcasting,
            TrainingState::Broadcasting => TrainingState::Collecting,
        }
    } 

    fn get_state(&self) -> &TrainingState {
        &self.state
    }

    fn run_state(&mut self){
    
        match self.state {
            TrainingState::Collecting =>{
                println!("Collecting gradients from all workers:");
                for wrkr in self.workers.iter_mut(){
                    let wrkr_cloned = Arc::clone(&wrkr);
                    let acc = Arc::clone(&self.accumulator);
                    thread::spawn(move || {
                        wrkr_cloned.lock().unwrap().compute_and_send(acc);
                    });
                    // wrkr.compute_and_send(&mut self.accumulator);
                }
            }

            TrainingState::Aggregating => {
                println!("Aggregating gradients in accumulator:");
                while !self.accumulator.lock().unwrap().is_ready(){
                    println!("Waiting for all gradients to be collected...");
                    thread::sleep(std::time::Duration::from_millis(1000));    
                }
                self.avg_grad = Some(self.accumulator.lock().unwrap().get_avg_gradient());
            }

            TrainingState::Updating => {

                println!("updating models from all workers:");
                if let Some(ref avg_g) = self.avg_grad {
                    for wrkr in self.workers.iter_mut(){
                        // println!("Avg Gradient used for update: {:?}", avg_g.gr_vec);
                        wrkr.lock().unwrap().update_model(avg_g);
                    }
                }
            }
            TrainingState::Broadcasting => {
                println!("Broadcasting updated models from all workers:");
                for wrkr in self.workers.iter(){
                    wrkr.lock().unwrap().broadcast_model();
                }
                self.accumulator.lock().unwrap().reset();
            }

        }

        self.transition_state();
    }

    pub fn run_cycles(&mut self){

        for _ in 0..10{
            for _ in 0..4 {
                self.run_state();
                println!("{}",self.get_state());
          }
        }
        
    }

}


// Collecting gradients -> wrokier computes and sends to accumulator
// Aggregating gradients -> accumulator computes average gradient
// update model -> worker updates model with average gradient
// broadcasting model -> worker broadcasts updated model (not implemented yet)