 
use crate::accumulator::Accumulator;
use crate::gradient::Gradient;
use crate::worker_pool::Threadpool;
use crate::worker::Worker;
use crate::paramer_server::ParamServer;
use std::fmt;
use std::thread;
use std::sync::{Arc, Mutex, mpsc};


enum TrainingState {
    Collecting,
    Aggregating,
    Updating, 
    Broadcasting,
}

pub struct TrainingCoordinator {
    state: TrainingState,
    // workers: Vec<Arc<Mutex<Worker>>>,
    worker_pool: Threadpool<Worker>,
    param_server_pool: Threadpool<ParamServer>,
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
        // let workers =vec![
        //     Arc::new(Mutex::new(Worker::new(vec![0.0; 10],1))),
        //     Arc::new(Mutex::new(Worker::new(vec![0.0; 10],2))),
        //     Arc::new(Mutex::new(Worker::new(vec![0.0; 10],3))),
        //     Arc::new(Mutex::new(Worker::new(vec![0.0; 10],4))),
        // ];

        let worker_pool = Threadpool::<Worker>::new(4, |id| {
            Worker::new(vec![0.0; 10], id)
        });

        let mut param_server_pool = Threadpool::<ParamServer>::new_empty();

        param_server_pool.add_worker(ParamServer::new(1, (0, 1000), String::from(""), mpsc::channel(), vec![]));
        param_server_pool.add_worker(ParamServer::new(2, (1001, 2000), String::from(""), mpsc::channel(), vec![]));



        let accumulator = Accumulator::new();

        TrainingCoordinator {
            state: TrainingState::Collecting,
            worker_pool,
            accumulator: Arc::new(Mutex::new(accumulator)),
            avg_grad: None,
            param_server_pool,
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
                for i in 0..self.worker_pool.get_num_workers() {
                    if let Some(wrkr) = self.worker_pool.get_worker_ref(i as u16) {
                        let acc = Arc::clone(&self.accumulator);
                        thread::spawn(move || {
                            wrkr.lock().unwrap().compute_and_send(acc);
                        });
                    }
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
                    for i in 0..self.worker_pool.get_num_workers() {
                        let wrkr = &self.worker_pool.get_worker_ref(i as u16).unwrap();
                        // println!("Avg Gradient used for update: {:?}", avg_g.gr_vec);
                        wrkr.lock().unwrap().update_model(avg_g);
                    }
                }
            }
            TrainingState::Broadcasting => {
                println!("Broadcasting updated models from all workers:");
            
                for i in 0..self.worker_pool.get_num_workers() {
                    if let Some(wrkr) = self.worker_pool.get_worker_ref(i as u16) {
                        wrkr.lock().unwrap().broadcast_model();
                    }
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