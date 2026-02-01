use std::thread;
use std::time::Duration;
use rand::Rng;

use crate::protocol::Protocol;
use crate::gradient::Gradient;  
use tokio::sync::broadcast::{Receiver, Sender};
use crate::worker_proxy::state::{compute, send, wait};
pub struct WorkerProxy {
    id: u16,
    tx: Sender<Protocol>,
    rx: Receiver<Protocol>,
    state: state,

    gradient_destinations: Vec<Protocol>
}


#[derive(Copy, Clone)] // better if eq or partial eq?
enum state {
    wait,
    compute,
    send,
}



impl WorkerProxy {
    pub fn new(id: u16, tx: Sender<Protocol>) -> Self {
        let mut rx = tx.subscribe();
        WorkerProxy { 
            id,
            tx,
            rx,
            state: wait,

            gradient_destinations: Vec::new()
        }
    }

    fn transition_state(&mut self) {
        match self.state {
            wait => self.state = compute,
            compute => self.state = send,
            send => self.state = wait
        }

    }

    fn is_state(&self, st: state) -> bool {
        match self.state {
            st => true,
            _ => false
        }
    }

    fn compute(&mut self){
        thread::sleep(Duration::from_secs(1));
        // this has to load the latest model and calculate the gradients for this step
    }

    fn send(&self){
        // implement the send of gradients to various param servers
        // for TCP maybe do TCP streaming.
    }

    pub async fn listen(&mut self){

        // This worker proxy thread should take two commands:
        // 1. Take param server addresses when provided (along with gradient range provided)
        // 2. take a command compute next set of gradients by loadng the present model,
        // The worker proxy should also send the gradients (sharded) to all the param server proxies
        // After sending the worker should wait for further commands

        let mut rx = self.tx.subscribe();
        loop {
            if let Ok(command) = rx.recv().await {
                match command {
                    Protocol::ToWorkerProxyCommandAddressChannelTcp {id, param_server_proxy_address, gradient_range} =>{
                        self.gradient_destinations.push(Protocol::ToWorkerProxyCommandAddressChannelTcp {id, param_server_proxy_address, gradient_range});
                    }

                    Protocol::ToWorkerProxyCommandAddressChannelLocal {id, param_server_proxy_address, gradient_range} => {
                        self.gradient_destinations.push(Protocol::ToWorkerProxyCommandAddressChannelLocal {id, param_server_proxy_address, gradient_range});
                    }

                    Protocol::ToWorkerProxyCommand {id, cmd} =>{

                        if cmd.eq("compute"){
                            if self.is_state(state::wait){
                                self.transition_state();
                                self.compute();
                            }
                            if self.is_state(state::send){
                                // this has to be rejected ?
                            }
                        }
                        else if cmd.eq("send"){
                            if self.is_state(state::compute){
                                self.transition_state();
                                self.send()
                            }
                            else if self.is_state(state::wait){
                                // reject it
                            }
                        }
                    }
                    
                    _ =>{
                        println!("Unknown command in worker proxy");
                    }
                }
            }
        }
    }
}