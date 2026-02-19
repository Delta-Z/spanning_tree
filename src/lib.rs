use rand::Rng;

pub mod algorithm;
pub mod graph;
mod messages;
mod node;
pub mod random_peer_generator;
pub mod tree_color;
pub mod tree_id;
mod tree_info;
pub mod ui;

#[derive(Debug)]
pub struct Configuration {
    n: usize,
    d: usize,
    epsilon: f64,
    initial_root_probablity: f64,
    id_range: u64,
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new(3, 2, 1.0, 0xffff)
    }
}

impl Configuration {
    pub fn new(n: usize, d: usize, initial_root_probablity: f64, id_range: u64) -> Self {
        assert!(d <= n, "d should not be larger than n");
        assert!(
            (0.0..=1.0).contains(&initial_root_probablity),
            "initial_root_probablity must be a probability"
        );
        Self {
            n,
            d,
            epsilon: (n as f64).recip(),
            initial_root_probablity,
            id_range,
        }
    }

    fn reset_countdown_max(&self) -> usize {
        // TODO: what should be the number?
        self.n
    }

    fn num_rounds_to_convergence(&self) -> usize {
        // TODO: what should be the number?
        self.n
    }

    fn max_color(&self) -> usize {
        // TODO: what should be the number?
        self.n
    }

    fn draw_from_initial_root_probablity(&self, rng: &mut impl Rng) -> bool {
        rng.random_bool(self.initial_root_probablity)
    }
}
