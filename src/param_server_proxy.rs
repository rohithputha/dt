
use crate::protocol::Protocol;
use std::sync::mpsc;
use tokio::sync::broadcast::{Sender , Receiver};
use crate::accumulator::Accumulator;

use crate::gradient::Gradient;

#[derive(Clone, Copy)]
enum state {
    compute,
    send,
    wait
}

pub struct ParamServerProxy {
    id: u16,
    rx: Receiver<Protocol>,
    accumulator: Accumulator,
    state: state,
    worker_address: Vec<Protocol>
}



impl ParamServerProxy {
    // each prama server proxy has its own accumulator
    pub fn new(id: u16, tx: Sender<Protocol>) ->  Self {
        let tx = tx;
        ParamServerProxy { 
            id,
            rx: tx.subscribe(),
            accumulator: Accumulator::new(),
            state: state::wait,
            worker_address: Vec::new(),
        }
    }

    fn transition_state(&mut self) {
        match self.state {
            state::wait => self.state = state::compute,
            state::compute => self.state = state::send,
            state::send => self.state = state::wait,
        }
    }

    fn is_state (&self, state: state)-> bool {
        match self.state {
            state=> true,
            _ => false
        }

    }
    
    fn compute(&self){
        
    }
    
    fn send(&self){
        
    }
    
    pub async fn listen(&mut self){
        // The param server proxy receives commands from the param server:
        // it receives command to accumulate
        // it receives command to wait and receive
        // it receives command to send avg gradients back

        // maybe recieve and send acks from the workwrs
        // this changes state from send to wait. similarly for proxy workers as well



        loop{
            if let Ok(command) = self.rx.recv().await {
                match command {
                    Protocol::ToParamServerProxyCommandAddressChannelLocal { id, worker_address } => {
                        self.worker_address.push(Protocol::ToParamServerProxyCommandAddressChannelLocal { id, worker_address });
                    },

                    Protocol::ToParamServerProxyCommandAddressChannelTcp {id, worker_address} =>{
                        self.worker_address.push(Protocol::ToParamServerProxyCommandAddressChannelTcp { id, worker_address });
                    },

                    Protocol::ToParamServerProxyCommand {id, cmd} =>{
                        if cmd.eq("compute") {
                            if self.is_state(state::wait){
                                self.state = state::compute;
                                self.compute();
                            }
                            else if self.is_state(state::send){
                                // reject
                            }
                        }
                        else if cmd.eq("send") {
                            if self.is_state(state::compute){
                                self.send();
                            } 
                            else if self.is_state(state::wait){
                                //reject
                            }
                        }
                    }

                    _ => {
                        println!("unknown command at param proxy ");
                    },
                }
            }
        }

    }
}