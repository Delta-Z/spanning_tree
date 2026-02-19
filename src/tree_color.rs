use crate::{Configuration, Rng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeColor {
    pub value: usize,
}

impl TreeColor {
    pub fn of(value: usize) -> Self {
        Self { value }
    }

    pub fn new_random(conf: &Configuration, rng: &mut impl Rng) -> Self {
        Self {
            value: rng.random_range(0..conf.max_color()),
        }
    }
}
