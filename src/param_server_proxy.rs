
use crate::protocol::Protocol;
use std::sync::mpsc;
pub struct ParamServerProxy {
    id: u16,
    rx: mpsc::Receiver<Protocol>,
}

impl ParamServerProxy {
    pub fn new(id: u16, rx: mpsc::Receiver<Protocol>) -> Self {
        ParamServerProxy { 
            id,
            rx
        }
    }

    pub fn listen(&self){
        loop{
            match self.rx.recv(){
                Ok(msg) =>{
                    println!("ParamServerProxy {} received a message", self.id);
                }
                Err(e)=>{
                    println!("ParamServerProxy {} error receiving message: {}", self.id, e);
                    break;
                }
            }
        }
    }
}