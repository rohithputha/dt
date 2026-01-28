
use rand::Rng;
use crate::accumulator::Accumulator;
use crate::gradient::Gradient;
use crate::worker_proxy::WorkerProxy;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

pub struct Worker {
    id: u16,
    dimension: u16,
    gr_vec: Vec<f32>,
    learn_rate: f32,
    model_vec: Vec<f32>,
    worker_proxy_tx: mpsc::Sender<u16>,
    // worker_proxy: Arc<WorkerProxy<u32>>,
}

impl Worker {


    pub fn new(model_vec: Vec<f32>, id: u16) -> Self {
        let rng = rand::thread_rng();
        let (tx, rx) = mpsc::channel::<u16>();
        let worker_proxy = WorkerProxy::<u16>::new(id as u16, rx);
        thread::spawn(move || {
            worker_proxy.listen();
        });
        Worker {
            id: id,
            dimension: 10, // for now hardcoding dimension to 10
            learn_rate: 0.01, // hardcoding learning rate to 0.01
            gr_vec: Vec::new(),
            model_vec: model_vec,
            worker_proxy_tx: tx,
        }
    }

    fn connect_to_proxy(&self){
        // future work: implement connection logic
    }

    pub fn compute_gradient(&mut self, step_number: u32) -> Gradient {
        
        let mut gr_vec_t = Vec::<f32>::new();
        let mut rng = rand::thread_rng();
        for _ in 0..self.dimension{
            gr_vec_t.push(rng.gen_range(-1.0..1.0));
        }

        self.gr_vec = gr_vec_t.clone();
        // self.perform_task();
        Gradient::new(gr_vec_t, 0, 0)

    }


    pub fn send_to_accumulator(&self, acc: Arc<Mutex<Accumulator>>){
        let grad = Gradient::new(self.gr_vec.clone(), 0, 0);
        println!("Worker {} sending gradient: {:?}", self.id, grad.gr_vec);
        acc.lock().unwrap().add_gradient(grad);
    }

    pub fn compute_and_send(&mut self, acc: Arc<Mutex<Accumulator>>){
        let grad = self.compute_gradient(0); // step_number is not used in this implementation
        self.send_to_accumulator(acc);
    }

    pub fn update_model(&mut self, avg_grad: &Gradient){
        println!("Worker {} updating model with gradient: {:?}", self.id, avg_grad.gr_vec);
        for (m, g) in self.model_vec.iter_mut().zip(avg_grad.gr_vec.iter()) {
            *m -= self.learn_rate * g;
        }
        // self.perform_task();
    }

    pub fn broadcast_model(&self){
        println!("Worker {} broadcasting model: {:?}", self.id, self.model_vec);
        println!("--------------------------------------------");
    } 
}