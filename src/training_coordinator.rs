 
use crate::accumulator::Accumulator;
use crate::gradient::Gradient;
use crate::worker_pool::Threadpool;
use crate::worker::Worker;
use crate::paramer_server::ParamServer;
use std::fmt;
use std::thread;
use std::sync::{Arc, Mutex, mpsc};
use crate::config_plane::ConfigPlane;
use crate::protocol::Protocol;
use tokio::sync::broadcast::{Sender, Receiver, channel};

enum TrainingState {
    Collecting,
    Aggregating,
    Updating, 
    Broadcasting,
}

struct w_state {
    ready: u16,
    not_ready: u16,
    total: u16,
}

pub struct TrainingCoordinator {
    state: TrainingState,
    // workers: Vec<Arc<Mutex<Worker>>>,
    worker_pool: Threadpool<Worker>,
    param_server_pool: Threadpool<ParamServer>,
    accumulator: Arc<Mutex<Accumulator>>,
    avg_grad: Option<Gradient>,

    config_plane_tx: Sender<Protocol>,
    config_plane_rx: Receiver<Protocol>,


    worker_state: w_state,
    param_server_state: w_state,
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

        let mut config_plane = ConfigPlane::new();


        let worker_pool = Threadpool::<Worker>::new(4, |id| {
            Worker::new(vec![0.0; 10], id)
        });

        let mut param_server_pool = Threadpool::<ParamServer>::new_empty();

        param_server_pool.add_worker(ParamServer::new(1, (0, 1000), String::from(""), mpsc::channel(), vec![]));  //need to fix this
        param_server_pool.add_worker(ParamServer::new(2, (1001, 2000), String::from(""), mpsc::channel(), vec![]));



        let accumulator = Accumulator::new();

        TrainingCoordinator {
            state: TrainingState::Collecting,
            worker_pool,
            accumulator: Arc::new(Mutex::new(accumulator)),
            avg_grad: None,
            param_server_pool,
            config_plane_tx : config_plane.get_config_plane_tx(),
            config_plane_rx : config_plane.subscribe(),
            worker_state :  w_state{
                ready: 0,
                not_ready: 0,
                total: 4,
            },
            param_server_state: w_state {
                ready: 0,
                not_ready: 0,
                total: 2,
            },
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

    async fn run_state(&mut self){
        // the coordinator receives messages from worker and paramserver threads
        // it receives the messages from the worker: in the following condition :
        // when all the worker are initiated, the worker sends a message
        // when each worker is done computing
        // when each worker is done sending
        // the coordinator receives messages from the param server
        // each param server sends a message when it receives when the param server receives the gradients
        // each param server sends a message when it sends back the gradients

        loop{
            if let Ok(command) = self.config_plane_rx.recv().await{
                match command {
                    Protocol::ToCoordinatorMessage {id, message, w_type} =>{
                        if w_type.eq("worker"){
                            if message.eq("wait"){
                                self.worker_state.ready+=1;
                                if self.worker_state.ready == self.worker_state.total {
                                    //set all workers to not ready
                                    // tell workers to start computing
                                }
                            }
                            else if message.eq("compute_done"){
                                // then tell the worker to send the gradients
                            }
                            else if message.eq("send_done"){
                                // then tell worker to wait
                                // make workers ready again
                            }
                        }

                        else if w_type.eq("param_server"){
                            if message.eq("wait") {
                                self.param_server_state.not_ready+=1;
                                if self.param_server_state.not_ready == self.param_server_state.total{
                                    // set the param server to ready
                                    // send to compute
                                }
                            }
                            if message.eq("compute_done"){
                                // send paramserver to send the gradients
                            }
                            if message.eq("send_done"){
                                // then make them wait
                                // set them non ready
                            }
                        }
                    },

                    _ => {
                        println!("not recognised command");
                    }
                }
            }
        }



    }

    pub fn run_cycles(&mut self){

        self.config_plane_tx.send(Protocol::ToParamServerCommand{ id: 0, cmd: "wait".to_string()}).unwrap();
        self.config_plane_tx.send(Protocol::ToWorkerCommand{id:1, cmd: "wait".to_string()}).unwrap();
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