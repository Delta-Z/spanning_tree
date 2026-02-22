use super::layout::{NodeCenterPoint, RootPositions, ViewMode};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    NextRound,
    ResizeGraph(usize),
    // Graph view settings:
    ViewMode(ViewMode),
    RootPositions(RootPositions),
    ShowTentativeRequests(bool),
    EditNode(usize, NodeCenterPoint),
    Animate,
}
