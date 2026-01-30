
use crate::protocol::Protocol;
use std::sync::mpsc;

use crate::accumulator::Accumulator;
use crate::gradient::Gradient;


enum ParamProxyState {
    Aggregate,
    Broadcast,
}

pub struct ParamServerProxy {
    id: u16,
    rx: mpsc::Receiver<Protocol>,
    accumulator: Accumulator,
    state: ParamProxyState,
}



impl ParamServerProxy {
    // each prama server proxy has its own accumulator
    pub fn new(id: u16, rx: mpsc::Receiver<Protocol>) ->  Self {
        ParamServerProxy { 
            id,
            rx,
            accumulator: Accumulator::new(),
            state: ParamProxyState::Aggregate,
        }
    }
    
    pub fn listen(&mut self){
        loop{
            match self.rx.recv(){
                Ok(command) =>{
                    match command {
                            Protocol::ParamServerProxyCommand{command_id, command_type}=>{
                                if command_type == "accumulate_gradient"{
                                    println!("ParamServerProxy {} received accumulate_gradient command: {}", self.id, command_id);
                                    self.state = ParamProxyState::Aggregate;
                                    
                                }
                                else if command_type == "broadcast_model"{
                                    println!("ParamServerProxy {} received broadcast_model command: {}", self.id, command_id);
                                    self.state = ParamProxyState::Broadcast;
                                    // future work: send the model to all the workers
                                }
                                
                            }

                            Protocol::ParamProxyWorkerGradientMessage{worker_id, gradient, gradient_range}=>{
                                match self.state {
                                    ParamProxyState::Aggregate=>{
                                        // proceed
                                    }
                                    ParamProxyState::Broadcast=>{
                                        println!("ParamServerProxy {} is in Broadcast state, cannot accept gradients now", self.id);
                                        continue;
                                    }
                                }
                                println!("ParamServerProxy {} received gradient from worker {}: {:?}", self.id, worker_id, gradient.gr_vec);
                                self.accumulator.add_gradient(gradient);
                                if self.accumulator.is_ready() {
                                    let avg_gradient = self.accumulator.get_avg_gradient(); // this has to be saved to the state of the proxy
                                    self.state = ParamProxyState::Broadcast;
                                }

                            }

                            _ => {
                                println!("ParamServerProxy {} received unknown protocol message", self.id);
                            }
                        }
                    }
                Err(e)=>{
                    println!("ParamServerProxy {} error receiving message: {}", self.id, e);
                    break;
                }


            }
        }
    }
}