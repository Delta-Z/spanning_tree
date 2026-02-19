use crate::graph::Graph;
use iced::{Pixels, Point, Size};

mod chord;
mod forest;

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
    fn node_radius(&self, viewport: Size) -> f32;
    fn arrange_nodes(&self, viewport: Size) -> Vec<NodeCenterPoint>;
}

pub fn graph_layout_for(
    graph: &Graph,
    view_mode: ViewMode,
    root_positions: RootPositions,
) -> impl GraphLayout + use<> {
    match view_mode {
        ViewMode::Chord => chord::ChordLayout::new(graph, root_positions),
        _ => panic!("Forest mode not implemented"),
    }
}

pub fn viewport_center(viewport: Size) -> Point {
    Point::new(viewport.width / 2.0, viewport.height / 2.0)
}

pub fn text_size(layout: &(impl GraphLayout + ?Sized), viewport: Size) -> Pixels {
    iced::Pixels(layout.node_radius(viewport) / 3.0)
}
