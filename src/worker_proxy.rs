use std::thread;
use rand::Rng;
use std::sync::mpsc;
use crate::protocol::Protocol;



enum param_server_config{
    local {
        param_server_id: u16,
        param_server_proxy_address: mpsc::Sender<Protocol>,
        gradient_range: (u128, u128),
    },

    remote {
        param_server_id: u16,
        ip_address: String,
        port: u16,
    }
}

enum state {
    compute,
    send,
}


pub struct WorkerProxy<T> {
    id: u16,
    rx: mpsc::Receiver<T>,

    param_servers: Vec<param_server_config>,
    
}

impl<T> WorkerProxy<T> {
    pub fn new(id: u16, rx: mpsc::Receiver<T>) -> Self {
        WorkerProxy { 
            id,
            rx,
            param_servers: Vec::new(),
        }
    }

    pub fn listen(&self){
        loop{
            match self.rx.recv(){
                Ok(command) =>{
                    match command {

                        
                        Protocol::ParamServerWorkerAddressChannelResponse{param_server_id,param_server_proxy_address,gradient_range} =>{
                            println!("WorkerProxy {} received ParamServerWorkerAddressChannelResponse from ParamServer {}", self.id, param_server_id);
                            // future work: save the proxy address and gradient range to send gradients later
                            self.param_servers.push(
                                param_server_config::local{
                                    param_server_id,
                                    param_server_proxy_address,
                                    gradient_range,
                                }
                            );
                        }


                    }
                }
                Err(e)=>{
                    println!("WorkerProxy {} error receiving message: {}", self.id, e);
                    break;
                }
            }
        }
    }

    pub fn perform_task(&self){

        let mut trg = rand::thread_rng();
        let sleep_time = trg.gen_range(1000..10000);
        thread::sleep(std::time::Duration::from_millis(sleep_time));
    
    }
}