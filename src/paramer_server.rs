

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


// For now, we implement a threaded param server resides on the same node as the coordinator and workers - future work: implement a distributed param server that can run on different nodes
// this design should be scalable to different nodes or even one node for all: each param server and each worker stay in one node along with coordinator. These threads talk to their proxies which handle the network communication. The coordinator manages the training process and coordinates between different param servers and workers. if there is only one node, the proxy is just another thread in the same node. If there are multiple nodes, param server and worker handle conncenctions to the proxy and tell what work to do and coordinate. 
use crate::accumulator::Accumulator;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use crate::protocol::Protocol;
use crate::param_server_proxy::ParamServerProxy;


pub struct ParamServer {
    id : u16,
    gradient_range : (u128, u128),
    worker_ids : Vec<u16>,
    proxy_address: String, // all these params should have another struct called ParamServerConfig

    tx: mpsc::Sender<Protocol>,
    worker_rx: mpsc::Receiver<Protocol>,
    proxy_tx: Option<mpsc::Sender<Protocol>>,
}


// this T has to be the protocol type?

impl ParamServer{
    pub fn new(id: u16, gradient_range: (u128, u128), proxy_address: String, (tx, rx): (mpsc::Sender<Protocol>, mpsc::Receiver<Protocol>)) -> Self {
        
        let mut paramServer = ParamServer{
            id: id,
            gradient_range: gradient_range,
            proxy_address: proxy_address,
            tx: tx,
            worker_rx: rx,
            worker_ids: Vec::new(),
            proxy_tx: None,
        };
        if ! paramServer.proxy_address.is_empty(){
            // future work: implement logic to get proxy address from coordinator
        }

        paramServer

    }
    
    // each worker proxy needs to send the worker to the params to all the param server proxies
    // worker server proxies get connected to the param server proxie themselves. 
    // Each worker requests the connection parameters from the coordinator (which param server threads handles and give the required info to the worker proxies)


    pub fn assign_worker(&mut self, worker_id: u16) -> bool {
        self.worker_ids.push(worker_id);
        true
    }
    pub fn connect_to_proxy(&mut self){
        if self.proxy_address.is_empty(){
            // spawn a proxy thread here // later can be changed to a seperate process
            let (tx, rx) = mpsc::channel::<Protocol>();
            self.proxy_tx = Some(tx.clone());
            let id = self.id;
            thread::spawn(move || {
                let proxy = ParamServerProxy::new(id,rx);
                proxy.listen()
            });
        }
        else{
            // future work: connect/spawn to a proxy process running at proxt address. 
        }
    }

    pub fn get_tx(&self) -> mpsc::Sender<Protocol> {
        self.tx.clone()
    }

    pub fn listen(&self) {
        loop {
            match self.worker_rx.recv(){
                Ok(req)=>{
                    match req {
                        Protocol::ParamServerRequest{request_id, request_type} => {
                            // handle param server request
                        }
                        _ => {
                            println!("ParamServer {} received unknown request", self.id);
                        }
                    }
                }
                Err(e)=>{
                    println!("ParamServer {} error receiving message: {}", self.id, e);
                    break;
                }
            }
        }
    }
}