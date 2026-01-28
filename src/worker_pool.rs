use std::sync::{Arc, Mutex};
use crate::worker::Worker;

pub struct Threadpool {
    workers: Vec<Arc<Mutex<Worker>>>,
}


// each worker will create a worker proxy: the worker proxy later on will try to connect with GPU or other worker nodes

// the coordinator will create a threadpool of workers, 
impl Threadpool {
    pub fn new(num: u16) -> Self {
        let mut workers = Vec::new();
        for i in 0..num{
            let worker = Arc::new(Mutex::new(Worker::new(vec![0.0; 10], i as u16)));
            workers.push(worker);
        }

        Threadpool {
            workers
        }
    }

    pub fn get_worker_ref(&self, id: u16) -> Option<Arc<Mutex<Worker>>> {
        if id as usize >= self.workers.len() {
            return None;
        }
        let wrkr = Arc::clone(&self.workers[id as usize]);
        Some(wrkr)
    }

    pub fn get_num_workers(&self) -> usize {
        self.workers.len()
    }


    // future work: implement a function to distribute tasks to workers in the pool
    // future work: implement a function to add or remove workers from the pool

}