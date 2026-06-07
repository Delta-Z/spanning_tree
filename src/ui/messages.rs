use super::layout::{RootPositions, ViewMode};
use crate::{graph::NodeIndex, tree_id::TreeId};
use iced::Size;

#[derive(Debug, Clone)]
pub enum TreeIdEdit {
    Valid(TreeId),
    Invalid(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    NextRound,
    // Graph configuration:
    ResizeGraph(usize),
    ChangeFanout(usize),
    // Graph view settings:
    ViewMode(ViewMode),
    RootPositions(RootPositions),
    ShowTentativeRequests(bool),
    // Graph editor:
    EditNode(NodeIndex, TreeIdEdit),
    UpdateBounds(Size),
    // Generic animation:
    Animate,
    // NoOp
    NoOp,
}
