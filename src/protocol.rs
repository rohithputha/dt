use crate::gradient::Gradient;

#[derive(Clone, Debug)]
pub enum Protocol {
    // GradientMessage{
    //     worker_id: u16,
    //     gradient: Gradient,
    // }

    // ParamServerRequest{
    //     request_id: u16,
    //     request_type: String,
    // },
    //
    // ParamServerProxyThreadReceiver{
    //     response_id: u16,
    //     tx: mpsc::Sender<Protocol>,
    // },
    //
    // ParamServerProxyCommand{
    //     command_id: u16,
    //     command_type: String, // future work: define proper payload types
    // },
    //
    // WorkerProxyCommand {
    //     command_id: u16,
    //     command_type: String,
    // },
    //
    // ParamServerWorkerAddressChannelResponse  {
    //     param_server_id: u16,
    //     param_server_proxy_address: mpsc::Sender<Protocol>,
    //     gradient_range: (u128, u128),
    // },
    //
    //
    // WorkerServerAddressChannelResponse {
    //     worker_id: u16,
    //     worker_proxy_address: mpsc::Sender<Protocol>,
    // },
    //
    // WorkerServerRequest{
    //     request_id: u16,
    //     request_type: String
    // },
    //
    // ParamProxyWorkerGradientMessage{
    //     worker_id: u16,
    //     gradient: Gradient,
    //     gradient_range: (u128, u128),
    // }

    ToWorkerCommandAddressChannelLocal{
        id: u16,
        param_server_proxy_address: String,
        gradient_range: (u128, u128),
    },
    ToWorkerCommandAddressChannelTcp{
        id: u16,
        param_server_proxy_address: Vec<String>,
        gradient_range: (u128, u128),
    },
    ToWorkerCommand{
        id: u16,
        cmd: String,
    },

    ToWorkerProxyCommandAddressChannelLocal{
        id: u16,
        param_server_proxy_address: String,
        gradient_range: (u128, u128),
    },
    ToWorkerProxyCommandAddressChannelTcp{
        id: u16,
        param_server_proxy_address: Vec<String>,
        gradient_range: (u128, u128),
    },

    ToWorkerProxyCommand {
        id: u16,
        cmd: String,
    },


    /////////////////////


    ToParamServerProxyCommand{
        id: u16,
        cmd:String,
    },
    ToParamServerProxyCommandAddressChannelLocal{
        id: u16,
        worker_address: String,
    },
    ToParamServerProxyCommandAddressChannelTcp {
        id: u16,
        worker_address: Vec<String>,
    },

    ToParamServerCommand{
        id: u16,
        cmd:String,
    },
    ToParamServerCommandAddressChannelLocal{
        id: u16,
        worker_address: String,
    },
    ToParamServerCommandAddressChannelTcp {
        id: u16,
        worker_address: Vec<String>,
    },

    ///////////////////////

    ToCoordinatorMessage{
        id: u16,
        message: String,
        w_type: String,
    },

    GradientToParamServer {
        worker_proxy_id: u16,
        gradient: Gradient,
    },
    GradientFromParamServer {
        param_server_proxy_id: u16,
        gradient: Gradient,
    },
}

