

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

pub struct ParamServer {
    id : u16,
    gradient_range : (u128, u128),
    worker_ids : Vec<u16>,
    proxy_streams: Vec<TcpStream>, // all these params should have another struct called ParamServerConfig// for now keeping it simple

    tx: mpsc::Sender<Protocol>,
    worker_rx: mpsc::Receiver<Protocol>,
    proxy_tx: Option<mpsc::Sender<Protocol>>, // there can be multiple proxies - future work: change this to a vector of tx

    worker_channels_tx: Vec<mpsc::Sender<Protocol>>, // channels to communicate with worker proxies
}


// this T has to be the protocol type?

impl ParamServer{
    pub fn new(id: u16, gradient_range: (u128, u128), proxy_address: String, (tx, rx): (mpsc::Sender<Protocol>, mpsc::Receiver<Protocol>), worker_channels_tx: Vec<mpsc::Sender<Protocol>>) -> Self {
        // the param server thread tries to connect with the proxy , should it be the other way round?
        let mut paramServer = ParamServer{
            id: id,
            gradient_range: gradient_range,
            tx: tx,
            worker_rx: rx,
            worker_ids: Vec::new(),
            proxy_tx: None,
            worker_channels_tx: worker_channels_tx,
            proxy_streams: Vec::new(),
        };
        paramServer.connect_to_proxy();

        paramServer

    }
    
    // each worker proxy needs to send the worker to the params to all the param server proxies
    // worker server proxies get connected to the param server proxie themselves. 
    // Each worker requests the connection parameters from the coordinator (which param server threads handles and give the required info to the worker proxies)


    pub fn add_worker(&mut self, worker_tx: mpsc::Sender<Protocol>) -> bool {
        self.worker_channels_tx.push(worker_tx);
        true
    }
    pub fn connect_to_proxy(&mut self){
        if self.proxy_streams.is_empty(){
            // spawn a proxy thread here // later can be changed to a seperate process
            // this can be done lazily when the first message needs to be sent to the proxy and no connection stream exists

            let (tx, rx) = mpsc::channel::<Protocol>();
            self.proxy_tx = Some(tx.clone());
            let id = self.id;
            thread::spawn(move || {
                let mut proxy = ParamServerProxy::new(id,rx);
                proxy.listen()
            });
        }
        else{
            // future work: connect/spawn to a proxy process running at proxy address. 
        }
    }

    pub fn get_tx(&self) -> mpsc::Sender<Protocol> {
        self.tx.clone()
    }


    pub fn listen(&self) {
        loop {
            // worker_rx is the receiver fot the message send to the proxy server

            match self.worker_rx.recv(){
                Ok(req)=>{
                    match req {
                        Protocol::ParamServerRequest{request_id, request_type} => {
                              if request_type  == "accumulate_gradient"{
                                  println!("ParamServer {} received accumulate_gradient request: {}", self.id, request_id);
                                  if let Some(proxy_tx) = &self.proxy_tx {
                                    proxy_tx.send(
                                    Protocol::ParamServerProxyCommand{
                                        command_id: request_id,
                                        command_type: String::from("accumulate_gradient"),
                        
                                        }
                                    );
                                  }
                                  
                                  // how to send back the addresses?
                                  // we need a vector of channels to the worker threads.
                                  for worker_tx in &self.worker_channels_tx {
                                      worker_tx.send(
                                        Protocol::ParamServerWorkerAddressChannelResponse{
                                            param_server_id: self.id,
                                            param_server_proxy_address: self.proxy_tx.clone().unwrap(),
                                            gradient_range: self.gradient_range,
                                        }
                                      ).unwrap();
                                    }
                                


                                  // this should give the address of the proxy to the send the gradients to and also the range of gradients expected
                                  // this can continue to update the model when all the gradients are received from all workers
                                  // this has to change the state of the param server proxy to accumulating gradients
                              }
                              else if request_type == "broadcast_model"{
                                  println!("ParamServer {} received broadcast_model request: {}", self.id, request_id);
                                  // this should send the command to all the proxies to send the updated model gradients to all the workers (along with the ranges), the input to the command being the address of workers to send to.
            
                              }
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