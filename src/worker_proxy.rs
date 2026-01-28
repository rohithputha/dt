use std::thread;
use rand::Rng;
use std::sync::mpsc;

pub struct WorkerProxy<T> {
    id: u16,
    rx: mpsc::Receiver<T>,
}

impl<T> WorkerProxy<T> {
    pub fn new(id: u16, rx: mpsc::Receiver<T>) -> Self {
        WorkerProxy { 
            id,
            rx
        }
    }

    pub fn listen(&self){
        loop{
            match self.rx.recv(){
                Ok(_) =>{
                    println!("WorkerProxy {} received a message", self.id);
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