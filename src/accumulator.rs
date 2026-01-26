use crate::gradient::Gradient;

pub struct Accumulator {
    pub grs: Vec<Gradient>,
    pub total_collected: u32,
}

impl Accumulator {
    pub fn new() -> Self {
        Accumulator {
            grs: Vec::new(),
            total_collected: 0,
        }
    }

    pub fn add_gradient(&mut self, grad: Gradient){
        self.grs.push(grad);
        self.total_collected += 1;
    }

    pub fn get_avg_gradient(&self)-> Gradient{
        let mut avg_grad = Vec::<f32>::new();
        let len = self.grs[0].gr_vec.len();
        for i in 0..len {
            let mut sum = 0.0;
            for g in self.grs.iter() {
                sum += g.gr_vec[i];
            }

            avg_grad.push(sum/(len as f32));
        }
        return Gradient::new(avg_grad, 0, 0);
        // maybe the change the step number to current step number;
    }

    pub fn is_ready(&self) -> bool {
        self.total_collected ==  4 // assuming 4 workers
    }

    pub fn reset(&mut self){
        self.grs.clear();
        self.total_collected = 0;
    }

}