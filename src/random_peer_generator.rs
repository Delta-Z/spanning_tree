use crate::graph::NodeIndex;
use crate::Rng;

pub struct RandomPeerGenerator<'a, R: Rng> {
    excluded: Vec<NodeIndex>,
    num_peers: usize,
    rng: &'a mut R,
}

pub trait PeerGenerator {
    fn generate_peer(&mut self) -> Option<NodeIndex>;
    fn exclude(&mut self, additional_excludes: Vec<NodeIndex>);
}

impl<'a, R: Rng> RandomPeerGenerator<'a, R> {
    pub fn new(num_peers: usize, mut excluded: Vec<NodeIndex>, rng: &'a mut R) -> Self {
        if !excluded.is_empty() {
            excluded.sort_unstable();
            let largest_excluded = *excluded.last().unwrap();
            assert!(
                largest_excluded < num_peers,
                "Invalid excluded element {} >= {}",
                largest_excluded,
                num_peers
            );
        }
        Self {
            excluded,
            num_peers,
            rng,
        }
    }
}

impl<R: Rng> PeerGenerator for RandomPeerGenerator<'_, R> {
    fn generate_peer(&mut self) -> Option<NodeIndex> {
        let remaining = self.num_peers as i64 - self.excluded.len() as i64;
        if remaining <= 0 {
            return None;
        }
        let mut result = self.rng.random_range(0..remaining as NodeIndex);
        for (i, excluded) in self.excluded.iter().enumerate() {
            if *excluded > result {
                self.excluded.insert(i, result);
                break;
            }
            result += 1;
        }
        if self.excluded.last().is_none_or(|x| *x < result) {
            self.excluded.push(result);
        }
        Some(result)
    }

    fn exclude(&mut self, additional_excludes: std::vec::Vec<NodeIndex>) {
        self.excluded.extend(additional_excludes);
        self.excluded.sort_unstable();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut rng = rand::rng();
        let mut g = RandomPeerGenerator::new(0, vec![], &mut rng);
        assert_eq!(g.generate_peer(), None);
    }

    #[test]
    fn generates_all() {
        let mut rng = rand::rng();
        let mut g = RandomPeerGenerator::new(5, vec![], &mut rng);
        let expected = vec![0, 1, 2, 3, 4];
        let mut generated: Vec<NodeIndex> = (0..expected.len())
            .map_while(|_| g.generate_peer())
            .collect();
        generated.sort_unstable();
        assert_eq!(generated, expected);
        assert_eq!(g.generate_peer(), None);
    }

    #[test]
    fn some_excluded() {
        let mut rng = rand::rng();
        let mut g = RandomPeerGenerator::new(10, vec![2, 3, 5, 7], &mut rng);
        let expected = vec![0, 1, 4, 6, 8, 9];
        let mut generated: Vec<NodeIndex> = (0..expected.len())
            .map_while(|_| g.generate_peer())
            .collect();
        generated.sort_unstable();
        assert_eq!(generated, expected);
        assert_eq!(g.generate_peer(), None);
    }

    #[test]
    #[should_panic(expected = "Invalid excluded element 3 >= 3")]
    fn invalid_exclude() {
        let mut rng = rand::rng();
        RandomPeerGenerator::new(3, vec![3], &mut rng);
    }
}
