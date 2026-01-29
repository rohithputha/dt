
use std::sync::mpsc;
pub enum Protocol {
    // GradientMessage{
    //     worker_id: u16,
    //     gradient: Gradient,
    // }

    ParamServerRequest{
        request_id: u16,
        request_type: String,
    },

    ParamServerProxyThreadReceiver{
        response_id: u16,
        tx: mpsc::Sender<Protocol>,
    }
}

impl Protocol {
    // pub fn new_gradient_message(worker_id: u16, gradient: Gradient)-> self{
    //     Protocol::GradientMessage{
    //         worker_id,
    //         gradient,
    //     }
    // }
}