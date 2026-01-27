
use rand::Rng;
use crate::accumulator::Accumulator;
use crate::gradient::Gradient;
use crate::worker_proxy::WorkerProxy;
use std::thread;
use std::sync::{Arc, Mutex};

pub struct Worker {
    id: u16,
    dimension: u16,
    gr_vec: Vec<f32>,
    learn_rate: f32,
    model_vec: Vec<f32>,

    worker_proxy: Arc<WorkerProxy>,
}

impl Worker {
    pub fn new(model_vec: Vec<f32>) -> Self {
        let mut rng = rand::thread_rng();
        let id = rng.gen_range(1..1000);
        Worker {
            id: id,
            dimension: 10, // for now hardcoding dimension to 10
            learn_rate: 0.01, // hardcoding learning rate to 0.01
            gr_vec: Vec::new(),
            model_vec: model_vec,
            worker_proxy: Arc::new(WorkerProxy::new(id as u16)),
        }
    }

    pub fn compute_gradient(&mut self, step_number: u32) -> Gradient {
        
        let mut gr_vec_t = Vec::<f32>::new();
        let mut rng = rand::thread_rng();
        for _ in 0..self.dimension{
            gr_vec_t.push(rng.gen_range(-1.0..1.0));
        }

        self.gr_vec = gr_vec_t.clone();
        self.perform_task();
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

     
    fn perform_task(&self){
        let wpx = Arc::clone(&self.worker_proxy);
        let handle = thread::spawn(move || {
            wpx.perform_task();
        });

        handle.join().unwrap();
    }
}