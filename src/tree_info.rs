use crate::tree_color::TreeColor;
use crate::tree_id::TreeId;
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeInfo {
    pub tree_id: TreeId,
    pub depth: usize,
    pub color: TreeColor,
    pub reset_countdown: usize,
}

impl Ord for TreeInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tree_id
            .cmp(&other.tree_id)
            .then(self.depth.cmp(&other.depth).reverse())
    }
}

impl PartialOrd for TreeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering() {
        let id1 = TreeId {
            primary: 10,
            secondary: 1,
        };
        let id2 = TreeId {
            primary: 10,
            secondary: 2,
        };
        let id3 = TreeId {
            primary: 5,
            secondary: 100,
        };
        let x = TreeInfo {
            tree_id: id1,
            depth: 0,
            color: TreeColor::of(100),
            reset_countdown: 0,
        };
        assert_eq!(x.cmp(&TreeInfo { tree_id: id2, ..x }), Ordering::Less);
        assert_eq!(x.cmp(&TreeInfo { tree_id: id3, ..x }), Ordering::Greater);
        assert_eq!(x.cmp(&TreeInfo { depth: 1, ..x }), Ordering::Greater);
        assert_eq!(
            x.cmp(&TreeInfo {
                color: TreeColor::of(10),
                reset_countdown: 5,
                ..x
            }),
            Ordering::Equal
        );
    }
}
