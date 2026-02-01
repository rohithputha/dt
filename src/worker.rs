
use rand::Rng;
use crate::accumulator::Accumulator;
use crate::gradient::Gradient;
use crate::worker_proxy::WorkerProxy;
use crate::protocol::Protocol;
use std::thread;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast::{Sender, Receiver, channel};

struct param_server_info {
    param_server_id: u16,
    param_server_proxy_address: Sender<Protocol>,
    gradient_range: (u128, u128),
}
pub struct Worker {
    id: u16,
    dimension: u16,
    gr_vec: Vec<f32>,
    learn_rate: f32,
    model_vec: Vec<f32>,

    // worker_proxy_tx: mpsc::Sender<Protocol>, // this has to be optional if we have multiple types of communication channels (think TCP, thread channels etc)

    // worker_tx: mpsc::Sender<Protocol>,
    // worker_rx:  mpsc::Receiver<Protocol>,
    // proxy_streams: Vec<TcpStream>, // a more robust way of choosing communication channels needed
    // param_server_channels: Vec<param_server_info>,
    // // worker_proxy: Arc<WorkerProxy<u32>>,

    config_plane_tx: Sender<Protocol>,
    config_plan_rx: Receiver<Protocol>,

    worker_proxy_address: Vec<String>,
    worker_proxy_true: bool,

}


// each worker will have to calculate the model m 
impl Worker {

    pub fn new(model_vec: Vec<f32>, id: u16, tx: Sender<Protocol>, proxy_address: Vec<String>) -> Self {
        let rng = rand::thread_rng();

        // spawning should be more conditional

        let rx = tx.subscribe();
        let mut worker = Worker {
            id: id,
            dimension: 10, // for now hardcoding dimension to 10
            learn_rate: 0.01, // hardcoding learning rate to 0.01
            gr_vec: Vec::new(),
            model_vec: model_vec,

            config_plane_tx: tx,
            config_plan_rx: rx,
            worker_proxy_address: Vec::new(),
            worker_proxy_true: false,
        };
        
        worker.connect_proxy();
        worker
    }

    pub async fn listen(&mut self){

        // The worker should take commands from the coordinator to
        // 1. start compute of the gradients
        // 2. send the gradients to the param servers
        // 3. maybe wait?



        loop{
            if let Ok(command) = self.config_plan_rx.recv().await {
                match command {

                    Protocol::ToWorkerCommandAddressChannelLocal{id, param_server_proxy_address, gradient_range} => {
                        self.config_plane_tx.send(Protocol::ToWorkerProxyCommandAddressChannelLocal {
                            id,
                            param_server_proxy_address,
                            gradient_range
                        }).unwrap();

                    }

                    Protocol::ToWorkerCommandAddressChannelTcp {id, param_server_proxy_address, gradient_range} => {
                        self.config_plane_tx.send(Protocol::ToWorkerProxyCommandAddressChannelTcp {
                            id,
                            param_server_proxy_address,
                            gradient_range
                        }).unwrap();
                    }

                    Protocol::ToWorkerCommand {id, cmd} =>{
                        // more filtering and intelligence needed here...
                        if cmd.eq("compute") || cmd.eq("send") {
                            self.config_plane_tx.send(Protocol::ToWorkerProxyCommand { id: id, cmd: cmd }).unwrap();
                        }
                    }



                    _ =>{
                        println!("command received by worker {}", self.id);
                        println!("command not processed by worker");
                    }


                }
            }

        }
    }

    pub fn connect_proxy(&self){
        if self.worker_proxy_address.len() != 0{
            // future work
        }
        else {
            // if there are no addresses prosent, then spawn the worker proxy thread.
            let mut worker_proxy = WorkerProxy::new(self.id, self.config_plane_tx.clone());
            thread::spawn(move || {
                worker_proxy.listen();
            });
        }
    }
}