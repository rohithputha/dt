use std::thread;
use rand::Rng;
use std::sync::mpsc;
use crate::protocol::Protocol;
use crate::gradient::Gradient;  

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
// more granualar states required later


pub struct WorkerProxy {
    id: u16,
    rx: mpsc::Receiver<Protocol>,

    param_servers: Vec<param_server_config>,
    state: state,
}

impl WorkerProxy {
    pub fn new(id: u16, rx: mpsc::Receiver<Protocol>) -> Self {
        WorkerProxy { 
            id,
            rx,
            param_servers: Vec::new(),
            state: state::compute,
        }
    }

    pub fn listen(&mut self){
        loop{
            match self.rx.recv(){
                Ok(command) =>{
                    match command {

                        Protocol::WorkerProxyCommand{command_id, command_type}=>{
                            if command_type == "compute"{
                                self.state = state::compute;

                                // do computation here
                                thread::sleep(std::time::Duration::from_millis(5000));
                            }
                            else if command_type == "send"{
                                self.state = state::send;
                                for param_server in &self.param_servers{
                                    match param_server {
                                        param_server_config::local{param_server_id, param_server_proxy_address, gradient_range}=>{
                                            println!("WorkerProxy {} sending gradient to ParamServer {}", self.id, param_server_id);
                                            // future work: send the actual gradient
                                            param_server_proxy_address.send(Protocol::ParamProxyWorkerGradientMessage{
                                                worker_id: self.id,
                                                gradient: Gradient::new(Vec::new(), 0, 0), // future work: send actual gradient
                                                gradient_range: *gradient_range,
                                            });
                                        }
                                        param_server_config::remote{param_server_id, ip_address, port}=>{
                                            println!("WorkerProxy {} cannot send gradient to remote ParamServer {} yet", self.id, param_server_id);
                                            // future work: implement sending to remote param server
                                        }
                                    }
                                }
                            }
                            // future work: handle different command types
                        }
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
                        Protocol::ParamProxyWorkerGradientMessage{worker_id, gradient, gradient_range}=>{
                            println!("WorkerProxy {} received ParamProxyWorkerGradientMessage from Worker {}", self.id, worker_id);
                            // future work: handle the received gradient
                        }

                        _ =>{
                            println!("WorkerProxy {} received unknown command", self.id);
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