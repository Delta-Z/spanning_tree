use crate::Rng;
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

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

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTreeIdError;

impl FromStr for TreeId {
    type Err = ParseTreeIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Result<Vec<u64>, Self::Err> = s
            .split(":")
            .map(|p| {
                usize::from_str_radix(p, 16)
                    .map_err(|_| ParseTreeIdError)
                    .and_then(|i| i.try_into().map_err(|_| ParseTreeIdError))
            })
            .collect();
        let parts = parts?;
        match parts.len() {
            1 => Ok(TreeId::new_simple(parts[0])),
            2 => Ok(TreeId {
                primary: parts[0],
                secondary: parts[1],
            }),
            _ => Err(ParseTreeIdError),
        }
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
