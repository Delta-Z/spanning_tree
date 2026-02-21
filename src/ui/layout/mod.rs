use crate::graph::Graph;
use iced::{Pixels, Point, Size};

mod chord;
mod forest;
pub mod transition;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ViewMode {
    Forest,
    Chord,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum RootPositions {
    Constant,
    Sorted,
}

pub type NodeCenterPoint = Point;

pub trait GraphLayout {
    fn node_radius(&self, viewport_size: Size) -> f32;
    fn arrange_nodes(&self, viewport_size: Size) -> Vec<NodeCenterPoint>;
}

pub fn graph_layout_for(
    graph: &Graph,
    view_mode: ViewMode,
    root_positions: RootPositions,
) -> Box<dyn GraphLayout> {
    match view_mode {
        ViewMode::Chord => Box::new(chord::ChordLayout::new(graph, root_positions)),
        ViewMode::Forest => Box::new(forest::ForestLayout::new(graph, root_positions)),
    }
}

pub fn viewport_center(viewport_size: Size) -> Point {
    Point::new(viewport_size.width / 2.0, viewport_size.height / 2.0)
}

pub fn text_size(layout: &(impl GraphLayout + ?Sized), viewport_size: Size) -> Pixels {
    iced::Pixels(layout.node_radius(viewport_size) / 3.0)
}
