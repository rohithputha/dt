

enum Protocol {
    GradientMessage{
        worker_id: u16,
        gradient: Gradient,
    }
}

impl Protocol {
    pub fn new_gradient_message(worker_id: u16, gradient: Gradient)-> self{
        Protocol::GradientMessage{
            worker_id,
            gradient,
        }
    }
}