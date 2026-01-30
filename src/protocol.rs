
use std::sync::mpsc;
use crate::gradient::Gradient;
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
    },
    
    ParamServerProxyCommand{
        command_id: u16,
        command_type: String, // future work: define proper payload types
    },

    WorkerProxyCommand {
        command_id: u16,
        command_type: String,
    },

    ParamServerWorkerAddressChannelResponse  {
        param_server_id: u16,
        param_server_proxy_address: mpsc::Sender<Protocol>,
        gradient_range: (u128, u128),
    },

    ParamProxyWorkerGradientMessage{
        worker_id: u16,
        gradient: Gradient,
        gradient_range: (u128, u128),
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