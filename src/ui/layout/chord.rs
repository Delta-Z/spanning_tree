use super::{viewport_center, GraphLayout, NodeCenterPoint, RootPositions};
use crate::graph::{Graph, NodeIndex};
use iced::Vector;
use iced::{Radians, Size};
use indexmap::IndexSet;
use itertools::Itertools;
use std::hash::RandomState;

pub struct ChordLayout {
    graph_to_chord_permutation: Vec<NodeIndex>,
}

impl GraphLayout for ChordLayout {
    fn arrange_nodes(&self, viewport_size: Size) -> Vec<NodeCenterPoint> {
        let num_nodes = self.graph_to_chord_permutation.iter().max().unwrap_or(&0) + 1;
        let mut result: Vec<Option<NodeCenterPoint>> = vec![None; num_nodes];
        for (chord_position, node_index) in self.graph_to_chord_permutation.iter().enumerate() {
            result[*node_index] = Some(chord_position_for_index(
                chord_position,
                num_nodes,
                viewport_size,
            ));
        }
        result
            .into_iter()
            .map(|p| p.expect("position not precomputed!"))
            .collect()
    }

    fn node_radius(&self, viewport_size: Size) -> f32 {
        let num_nodes = self.graph_to_chord_permutation.len();
        if num_nodes == 1 {
            chord_radius(viewport_size)
        } else {
            let distance_between_nodes = chord_position_for_index(0, num_nodes, viewport_size)
                .distance(chord_position_for_index(1, num_nodes, viewport_size));
            // (Radians::PI * 2.0 / graph.nodes().len() as f32).0.min(1.0) * chord_radius(viewport_size);
            let chord_margin =
                viewport_size.height.min(viewport_size.width) / 2.0 - chord_radius(viewport_size);
            (distance_between_nodes * 0.35).min(chord_margin)
        }
    }
}

impl ChordLayout {
    pub fn new(graph: &Graph, root_positions: RootPositions) -> Self {
        let num_nodes = graph.nodes().len();
        Self {
            graph_to_chord_permutation: match root_positions {
                RootPositions::Constant => (0..num_nodes).collect_vec(),
                RootPositions::Sorted => {
                    let mut result: IndexSet<NodeIndex> = graph
                        .trees(/*confirmed_only=*/ false)
                        .into_iter()
                        .sorted_by_key(|t| -(t.nodes().len() as i64))
                        .flatten()
                        .collect(); // Tree nodes
                    result.append::<RandomState>(&mut (0..num_nodes).collect());
                    result.into_iter().collect()
                }
            },
        }
    }
}

fn chord_position_for_index(i: usize, n: usize, viewport_size: Size) -> NodeCenterPoint {
    if n == 1 {
        return viewport_center(viewport_size);
    }
    let angle = Radians::PI * 2.0 * (i as f32 / n as f32)
        // Start from the top of the viewport_size
        + Radians::PI;
    let radius = chord_radius(viewport_size);
    viewport_center(viewport_size) + Vector::new(radius * angle.0.sin(), radius * angle.0.cos())
}

fn chord_radius(viewport_size: Size) -> f32 {
    viewport_size.width.min(viewport_size.height) / 2.4
}
