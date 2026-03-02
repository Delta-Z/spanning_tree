use crate::graph::NodeIndex;
use crate::tree_color::TreeColor;
use crate::tree_info::TreeInfo;

#[derive(Clone, Debug)]
pub enum Message {
    Request(TreeInfo),
    Confirmation(Option<TreeColor>),
}

#[derive(Clone, Debug)]
pub struct SentMessage {
    pub destination: NodeIndex,
    pub message: Message,
}

#[derive(Clone, Debug)]
pub struct ReceivedMessage {
    pub source: NodeIndex,
    pub message: Message,
}
