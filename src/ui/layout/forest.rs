use itertools::Itertools;

use super::GraphLayout;
use crate::graph::{Graph, Tree};
use crate::ui::RootPositions;

pub struct ForestLayout {
    trees: Vec<TreeInfo>,
}

struct TreeInfo {
    root: usize,
    height: usize,
    width: usize,
}

impl From<Tree<'_>> for TreeInfo {
    fn from(tree: Tree<'_>) -> Self {
        Self {
            root: tree.root(),
            height: tree.height(),
            width: tree_width(&tree),
        }
    }
}

impl GraphLayout for ForestLayout {
    fn arrange_nodes(&self, _: iced::Size) -> std::vec::Vec<iced::Point> {
        todo!()
    }
    fn node_radius(&self, _: iced::Size) -> f32 {
        todo!()
    }
}

impl ForestLayout {
    pub fn new(graph: &Graph, root_positions: RootPositions) -> Self {
        let mut trees = graph.trees();
        if root_positions == RootPositions::Sorted {
            trees.sort_by_key(|t| -(t.nodes().len() as i64));
        }

        Self {
            trees: trees.into_iter().map_into().collect(),
        }
    }
}

fn tree_width<'a>(tree: &Tree<'a>) -> usize {
    let num_nodes_per_depth = tree
        .nodes()
        .iter()
        .counts_by(|i| tree.graph().nodes()[*i].parenting().my_depth());
    num_nodes_per_depth.into_values().max().unwrap()
}
