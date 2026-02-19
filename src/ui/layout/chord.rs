use super::{viewport_center, GraphLayout, NodeCenterPoint, RootPositions};
use crate::graph::Graph;
use iced::Vector;
use iced::{Radians, Size};
use indexmap::IndexSet;
use itertools::Itertools;
use std::hash::RandomState;

pub struct ChordLayout {
    graph_to_chord_permutation: Vec<usize>,
}

impl GraphLayout for ChordLayout {
    fn arrange_nodes(&self, viewport: Size) -> Vec<NodeCenterPoint> {
        let num_nodes = self.graph_to_chord_permutation.iter().max().unwrap_or(&0) + 1;
        let mut result: Vec<Option<NodeCenterPoint>> = vec![None; num_nodes];
        for (chord_position, node_index) in self.graph_to_chord_permutation.iter().enumerate() {
            result[*node_index] = Some(chord_position_for_index(
                chord_position,
                num_nodes,
                viewport,
            ));
        }
        result
            .into_iter()
            .map(|p| p.expect("position not precomputed!"))
            .collect()
    }

    fn node_radius(&self, viewport: Size) -> f32 {
        let num_nodes = self.graph_to_chord_permutation.len();
        if num_nodes == 1 {
            chord_radius(viewport)
        } else {
            let distance_between_nodes = chord_position_for_index(0, num_nodes, viewport)
                .distance(chord_position_for_index(1, num_nodes, viewport));
            // (Radians::PI * 2.0 / graph.nodes().len() as f32).0.min(1.0) * chord_radius(viewport);
            let chord_margin = viewport.height.min(viewport.width) / 2.0 - chord_radius(viewport);
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
                    let mut result: IndexSet<usize> = graph
                        .trees()
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

fn chord_position_for_index(i: usize, n: usize, viewport: Size) -> NodeCenterPoint {
    if n == 1 {
        return viewport_center(viewport);
    }
    let angle = Radians::PI * 2.0 * (i as f32 / n as f32);
    let radius = chord_radius(viewport);
    viewport_center(viewport) + Vector::new(radius * angle.0.sin(), radius * angle.0.cos())
}

fn chord_radius(viewport: Size) -> f32 {
    viewport.width.min(viewport.height) / 2.4
}
