use super::layout::{RootPositions, ViewMode};
use crate::graph::NodeIndex;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    NextRound,
    ResizeGraph(usize),
    // Graph view settings:
    ViewMode(ViewMode),
    RootPositions(RootPositions),
    ShowTentativeRequests(bool),
    // Graph editor:
    EditNode(NodeIndex),
    // Generic animation:
    Animate,
}
