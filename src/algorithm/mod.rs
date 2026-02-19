use crate::Configuration;
use crate::Rng;

pub mod coloring;
pub mod parenting;
pub mod timers;

pub trait RandomizableData {
    fn new_random(conf: &Configuration, rng: &mut impl Rng) -> Self;
    fn reset(&mut self, conf: &crate::Configuration, rng: &mut impl rand::Rng);
}
