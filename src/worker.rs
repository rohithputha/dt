
use rand::Rng;
use crate::accumulator::Accumulator;
use crate::gradient::Gradient;

pub struct Worker {
    id: u32,
    dimension: u16,
    gr_vec: Vec<f32>,
    learn_rate: f32,
    model_vec: Vec<f32>,
}

impl Worker {
    pub fn new(model_vec: Vec<f32>) -> Self {
        let mut rng = rand::thread_rng();
        Worker {
            id: rng.gen_range(1..1000),
            dimension: 10, // for now hardcoding dimension to 10
            learn_rate: 0.01, // hardcoding learning rate to 0.01
            gr_vec: Vec::new(),
            model_vec: model_vec,
        }
    }

    pub fn compute_gradient(&mut self, step_number: u32) -> Gradient {
        //similating gradient computation
        let mut gr_vec_t = Vec::<f32>::new();
        let mut rng = rand::thread_rng();
        for _ in 0..self.dimension{
            gr_vec_t.push(rng.gen_range(-1.0..1.0));
        }

        self.gr_vec = gr_vec_t.clone();
        Gradient::new(gr_vec_t, 0, 0)
    }


    pub fn send_to_accumulator(&self, acc: &mut Accumulator){
        let grad = Gradient::new(self.gr_vec.clone(), 0, 0);
        acc.add_gradient(grad);
    }

    pub fn compute_and_send(&mut self, acc: &mut Accumulator){
        let grad = self.compute_gradient(0); // step_number is not used in this implementation
        self.send_to_accumulator(acc);
    }

    pub fn update_model(&mut self, avg_grad: &Gradient){
        println!("Worker {} updating model with gradient: {:?}", self.id, avg_grad.gr_vec);
        println!("<><><><>");
        for (m, g) in self.model_vec.iter_mut().zip(avg_grad.gr_vec.iter()) {
            *m -= self.learn_rate * g;
        }
    }

    pub fn broadcast_model(&self){
        println!("Worker {} broadcasting model: {:?}", self.id, self.model_vec);
        println!("--------------------------------------------");
    } 

    // fn train(&mut self){
    //     // simulate train
    //     self.model_vec.clear();
    //     for _ in 0..self.dimension{
    //         self.model_vec.push(rand::thread_rng().gen_range(-10.0..10.0));
    //     }


    // }
}