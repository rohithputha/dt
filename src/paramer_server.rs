

// the param server receieves a set of gradients from workers, accumulate them (using an Accumulator), compute the average gradient, update the global model, and broadcast the updated model to all workers
// There are some guarantees that need to be ensured:
// 1. The param server should only update the global model after receiving gradients from all workers
// 2. The different parameter server should update the global model in a synchronized manner
// 3. The param server should be able to handle failure of workers and other param servers (maybe implement quorum based updates - but to what extent?)
// 4. The param server should be able to handle straggler workers (maybe implement timeouts and use the gradients received so far to update the model?)
// 5. The param server should be able to scale to a large number of workers and param servers (maybe implement sharding of the model and distribute the shards across multiple param servers?)
// 6. The param server should be able to handle different types of models (maybe implement a generic model interface that can be used by different types of models?)
// 7. The param server should be able to handle different types of gradients (maybe implement a generic gradient interface that can be used by different types of gradients?)
// 8. The param server should be able to handle different types of optimizers (maybe implement a generic optimizer interface that can be used by different types of optimizers?)
// 9. The param server should be able to handle different types of loss functions (maybe implement a generic loss function interface that can be used by different types of loss functions?)
// 10. The param server is not he coordinator - the coordinator will manage the training process and coordinate between different param servers and workers
// 11. Each param server thread and even the worker thread should have a enpoint to the network where they can recieve commands from remote coordinator -> should this be on the proxies?


// For now, we implement a threaded param server resides on the same node as the coordinator and workers - future work: implement a distributed param server that can run on different nodes
// this design should be scalable to different nodes or even one node for all: each param server and each worker stay in one node along with coordinator. These threads talk to their proxies which handle the network communication. The coordinator manages the training process and coordinates between different param servers and workers. if there is only one node, the proxy is just another thread in the same node. If there are multiple nodes, param server and worker handle conncenctions to the proxy and tell what work to do and coordinate. 
use crate::accumulator::Accumulator;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use crate::protocol::Protocol;
use crate::param_server_proxy::ParamServerProxy;
use std::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Sender, Receiver};



pub struct ParamServer {
    id : u16,
    gradient_range : (u128, u128),
    worker_ids : Vec<u16>,

    tx: Sender<Protocol>,
    proxy_address: Vec<String>
}


// this T has to be the protocol type?

impl ParamServer{
    pub fn new(id: u16, gradient_range: (u128, u128), tx: Sender<Protocol>, proxy_address: Vec<String>) -> Self {
        // the param server thread tries to connect with the proxy , should it be the other way round?
        let mut paramServer = ParamServer{
            id: id,
            gradient_range: gradient_range,
            tx: tx,
            worker_ids: Vec::new(),
            proxy_address: proxy_address,

        };

        paramServer.connect_to_proxy();

        paramServer
    }


    pub fn connect_to_proxy(&mut self){
        if self.proxy_address.is_empty(){
            // spawn a proxy thread here // later can be changed to a separate process
            // this can be done lazily when the first message needs to be sent to the proxy and no connection stream exists

            let id = self.id;
            let tx = self.tx.clone();
            thread::spawn(move || {
                let mut proxy = ParamServerProxy::new(id,tx);
                proxy.listen();
            });
        }
        else{
            // future work: connect/spawn to a proxy process running at proxy address. 
        }
    }

    pub async fn listen(&self) {
        // the param server thread listens to the the coordinator for the following commands:
        // receive the worker server address
        // send compute adn send commands to the proxy server
        let mut rx = self.tx.subscribe();
        loop{
            if let Ok(command) = rx.recv().await{
                match command {
                    Protocol::ToParamServerCommand{id, cmd} =>{
                        if cmd.eq("compute") || cmd.eq("send"){
                            self.tx.send(Protocol::ToParamServerProxyCommand {
                                id,
                                cmd,
                            }).unwrap();
                        }
                    }
                    Protocol::ToParamServerCommandAddressChannelLocal {id, worker_address}=>{
                        self.tx.send(Protocol::ToParamServerProxyCommandAddressChannelLocal {
                            id,
                            worker_address,
                        })
                        .unwrap();
                    }
                    Protocol::ToParamServerCommandAddressChannelTcp {id, worker_address}=>{
                        self.tx.send(Protocol::ToParamServerProxyCommandAddressChannelTcp {
                            id,
                            worker_address,
                        }).unwrap();
                    }
                    _=>{
                        println!("Command not recognized");
                    }
                }
            }
        }
    }
}