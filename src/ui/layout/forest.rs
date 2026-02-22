#![allow(unused)]

use iced::{Padding, Rectangle, Size};
use itertools::Itertools;

use super::{GraphLayout, NodeCenterPoint};
use crate::graph::Graph;
use crate::ui::RootPositions;

pub struct ForestLayout {
    total_nodes: usize,
    trees: Vec<Tree>,
}

type TreeLevel = Vec<usize>;

struct Tree {
    levels: Vec<TreeLevel>,
}

impl Tree {
    fn new(root: usize, graph: &Graph) -> Self {
        let mut result = Self {
            levels: vec![vec![root]],
        };
        loop {
            let next_level = result
                .levels
                .last()
                .unwrap()
                .iter()
                .flat_map(|i| graph.nodes()[*i].parenting().confirmed_children())
                .copied()
                .collect_vec();
            if next_level.is_empty() {
                break;
            }
            result.levels.push(next_level);
        }
        result
    }

    fn max_width(&self) -> usize {
        self.levels.iter().map(|l| l.len()).max().unwrap()
    }

    fn levels(&self) -> &[TreeLevel] {
        &self.levels
    }
}

impl GraphLayout for ForestLayout {
    fn arrange_nodes(&self, viewport_size: Size) -> Vec<NodeCenterPoint> {
        let mut result: Vec<Option<NodeCenterPoint>> = vec![None; self.total_nodes];
        let tree_widths = self.trees.iter().map(|t| t.max_width()).collect_vec();
        let horizontal_spacing = viewport_size.width / tree_widths.iter().sum::<usize>() as f32;
        let padding = horizontal_spacing / 2.0;
        let mut remaining_viewport = Rectangle::with_size(viewport_size);
        for (tree, max_width) in self.trees.iter().zip_eq(tree_widths) {
            remaining_viewport = remaining_viewport.shrink(Padding::ZERO.left(padding));
            let tree_viewport = Rectangle::new(
                remaining_viewport.position(),
                Size::new(
                    (max_width - 1) as f32 * horizontal_spacing,
                    remaining_viewport.height,
                ),
            );
            if tree_viewport.width > remaining_viewport.width {
                println!("not enough width to render!")
            }
            self.arrange_tree(tree, tree_viewport, &mut result);
            remaining_viewport =
                remaining_viewport.shrink(Padding::ZERO.left(tree_viewport.width + padding));
        }
        result
            .into_iter()
            .enumerate()
            .map(|(i, mp)| {
                if let Some(p) = mp {
                    p
                } else {
                    panic!("No location computed for {i}")
                }
            })
            .collect()
    }

    fn node_radius(&self, viewport_size: Size) -> f32 {
        viewport_size.width.min(viewport_size.height) / (self.total_nodes as f32).sqrt() / 4.0
    }
}

impl ForestLayout {
    pub fn new(graph: &Graph, root_positions: RootPositions) -> Self {
        let mut trees = graph.trees();
        if root_positions == RootPositions::Sorted {
            trees.sort_by_key(|t| -(t.nodes().len() as i64));
        }
        Self {
            total_nodes: graph.nodes().len(),
            trees: trees
                .into_iter()
                .map(|t| Tree::new(t.root(), graph))
                .collect(),
        }
    }

    fn arrange_tree(
        &self,
        tree: &Tree,
        mut viewport: Rectangle,
        output: &mut [Option<NodeCenterPoint>],
    ) {
        let vertical_spacing = viewport.height / (tree.levels.len()) as f32;
        viewport = viewport.shrink(Padding::ZERO.top(vertical_spacing / 2.0));
        let vertical_spacing = Padding::ZERO.top(vertical_spacing);
        for level in &tree.levels {
            arrange_level(level, viewport, output);
            viewport = viewport.shrink(vertical_spacing);
        }
    }
}

fn arrange_level(
    tree_level: &TreeLevel,
    viewport: Rectangle,
    output: &mut [Option<NodeCenterPoint>],
) {
    if tree_level.len() == 1 {
        output[*tree_level.last().unwrap()] =
            Some(NodeCenterPoint::new(viewport.center_x(), viewport.y));
        return;
    }
    let spacing = viewport.width / (tree_level.len() - 1) as f32;
    let mut x = viewport.x;
    for i in tree_level {
        output[*i] = Some(NodeCenterPoint::new(x, viewport.y));
        x += spacing;
    }
}
