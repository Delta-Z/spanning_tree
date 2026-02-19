use crate::Rng;
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TreeId {
    pub primary: u64,
    pub secondary: u64,
}

impl Ord for TreeId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.primary
            .cmp(&other.primary)
            .then(self.secondary.cmp(&other.secondary))
    }
}

impl PartialOrd for TreeId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for TreeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:X}:{:x}", self.primary, self.secondary)
    }
}

impl TreeId {
    pub fn new_simple(primary: u64) -> Self {
        Self {
            primary,
            secondary: 0,
        }
    }

    pub fn new(epsilon: f64, rng: &mut impl Rng) -> Self {
        let secondary_key_max: u64 = ((36.0 * (4.0 / epsilon).log2() / epsilon) as u64).max(1);
        Self {
            primary: Self::generate_geometric(rng),
            secondary: rng.random_range(0..secondary_key_max),
        }
    }

    pub fn new_random(max: u64, rng: &mut impl Rng) -> Self {
        Self {
            primary: rng.random_range(0..max),
            secondary: rng.random_range(0..max),
        }
    }

    fn generate_geometric(rng: &mut impl Rng) -> u64 {
        let value = -rng.sample::<f32, _>(rand::distr::OpenClosed01).log2();
        value.ceil() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_simple() {
        assert_eq!(
            TreeId::new_simple(1001),
            TreeId {
                primary: 1001,
                secondary: 0
            }
        );
    }

    #[test]
    fn ordering() {
        let x = TreeId::new_simple(1);
        assert_eq!(x.cmp(&TreeId { primary: 0, ..x }), Ordering::Greater);
        assert_eq!(x.cmp(&TreeId { primary: 2, ..x }), Ordering::Less);
        assert_eq!(x.cmp(&TreeId { secondary: 1, ..x }), Ordering::Less);
    }
}
