use std::sync::{Arc, Mutex};

pub struct Threadpool<T> {
    workers: Vec<Arc<Mutex<T>>>,
}

impl<T> Threadpool<T> {
    pub fn new(num: u16, factory: fn(u16)-> T) -> Self {
        let mut workers = Vec::new();
        for i in 0..num{
            let worker = Arc::new(Mutex::new(factory(i)));
            workers.push(worker);
        }

        Threadpool {
            workers
        }
    }

    pub fn new_empty()-> Self {
        Threadpool {
            workers: Vec::new(),
        }

    }

    pub fn get_worker_ref(&self, id: u16) -> Option<Arc<Mutex<T>>> {
        if id as usize >= self.workers.len() {
            return None;
        }
        let wrkr = Arc::clone(&self.workers[id as usize]);
        Some(wrkr)
    }

    pub fn get_num_workers(&self) -> usize {
        self.workers.len()
    }

    pub fn add_worker(&mut self, worker: T) {
        let wrkr = Arc::new(Mutex::new(worker));
        self.workers.push(wrkr);
    }

    


    // future work: implement a function to distribute tasks to workers in the pool
    // future work: implement a function to add or remove workers from the pool

}