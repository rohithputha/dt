use std::thread;
use rand::Rng;
pub struct WorkerProxy {
    id: u16,
}

impl WorkerProxy {
    pub fn new(id: u16) -> Self {
        WorkerProxy { 
            id
        }
    }

    pub fn perform_task(&self){
        let mut trg = rand::thread_rng();
        let sleep_time = trg.gen_range(1000..10000);
        thread::sleep(std::time::Duration::from_millis(sleep_time));
    }
}