#[derive(Clone, Debug)]
pub struct Gradient {
    pub gr_vec: Vec<f32>,
    pub step_number: u32,
    timestamp: u128,
}

impl Gradient {
    pub fn new(gr_vec: Vec<f32>, wrk_id: u32, step_number: u32) -> Self {
        Gradient {
            gr_vec,
            step_number,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }

    pub fn add_vec(&mut self, other: &Gradient){
        for (a, b) in self.gr_vec.iter_mut().zip(other.gr_vec.iter()) {
            *a += b
        }
    }

    pub fn scale_vec(&mut self, scalar: f32){
        for a in self.gr_vec.iter_mut(){
            *a = *a * scalar;
        }
    }


}