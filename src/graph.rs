use crate::algorithm::RandomizableData;
use crate::messages::ReceivedMessage;
use crate::node::Node;
use crate::tree_color::TreeColor;
use crate::tree_id::TreeId;
use crate::Configuration;
use rand::Rng;
use std::fmt;

pub type NodeIndex = usize;
type ReceivedMessages = Vec<ReceivedMessage>;
type MessageRouting = Vec<ReceivedMessages>;

#[derive(Debug)]
pub struct Graph {
    conf: Configuration,
    nodes: Vec<Node>,
    messages: MessageRouting,
}

#[derive(Debug)]
pub struct Tree<'a> {
    nodes: Vec<NodeIndex>,
    graph: &'a Graph,
}

impl Tree<'_> {
    pub fn new<'a>(graph: &'a Graph, nodes: Vec<NodeIndex>) -> Tree<'a> {
        assert!(!nodes.is_empty(), "Tree cannot be empty!");
        Tree::<'a> { nodes, graph }
    }

    pub fn root(&self) -> NodeIndex {
        self.nodes[0]
    }

    pub fn nodes(&self) -> &[NodeIndex] {
        &self.nodes
    }

    pub fn height(&self) -> usize {
        self.nodes
            .iter()
            .map(|i| self.graph.nodes[*i].parenting().my_depth())
            .max()
            .unwrap()
            + 1
    }
}

impl fmt::Display for Tree<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_nodes = self.nodes.len();
        let root = self.root();
        write!(
            f,
            "tree of {} node{} rooted at [{}] {}",
            num_nodes,
            if num_nodes == 1 { "" } else { "s" },
            root,
            self.graph.nodes[root],
        )
    }
}

impl std::iter::IntoIterator for Tree<'_> {
    type Item = NodeIndex;
    type IntoIter = std::vec::IntoIter<NodeIndex>;

    fn into_iter(self) -> <Self as std::iter::IntoIterator>::IntoIter {
        self.nodes.into_iter()
    }
}

impl Graph {
    pub fn new_random(conf: Configuration, rng: &mut impl Rng) -> Self {
        let mut g = Graph {
            messages: no_messages(&conf),
            conf,
            nodes: Vec::new(),
        };
        g.nodes = core::iter::from_fn(|| Some(Node::new_random(&g.conf, rng)))
            .take(g.conf.n)
            .collect();
        g
    }

    pub fn new_test(nodes: Vec<(TreeId, TreeColor)>, d: usize) -> Self {
        let conf = Configuration::new(
            nodes.len(),
            d,
            0.0,
            nodes
                .iter()
                .map(|(id, _)| id.primary.max(id.secondary))
                .max()
                .unwrap_or_default(),
        );
        let nodes = nodes
            .into_iter()
            .map(|(id, color)| Node::new_root(&conf, id, color))
            .collect();
        Graph { messages: no_messages(&conf), conf, nodes }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn configuration(&self) -> &Configuration {
        &self.conf
    }

    pub fn resize(&mut self, new_size: usize, rng: &mut impl Rng) {
        assert!(new_size > 0, "cannot resize graph below 1");
        let downsized = new_size < self.conf.n;
        self.conf.n = new_size;
        self.nodes
            .resize_with(new_size, || Node::new_random_root(&self.conf, rng));
        self.messages.resize(new_size, vec![]);
        if downsized {
            self.nodes
                .iter_mut()
                .for_each(|n| n.update_for_configuration(&self.conf));
            for messages in &mut self.messages {
                messages.retain(|m| m.source < new_size);
            }
        }
    }

    pub fn trees<'a>(&'a self, confirmed_only: bool) -> Vec<Tree<'a>> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if let Some(p) = n.parenting().parent()
                        && (!confirmed_only || self.nodes[p].parenting().children(confirmed_only).contains(&i))
                {
                    None
                } else {
                    Some(Tree::new(self, self.subtree(i, confirmed_only)))
                }
            })
            .collect()
    }

    pub fn execute_round(&mut self, rng: &mut impl Rng) {
        self.receive_messages(rng);
        self.nodes.iter_mut().for_each(|n| n.advance_time(&self.conf, rng));
        self.send_messages();
    }

    pub fn in_flight_messages(&self) -> &[ReceivedMessages] {
        &self.messages
    }

    pub fn edit_node(&mut self, index: NodeIndex) -> &mut Node {
        &mut self.nodes[index]
    }

    pub fn validate_parenting(&self, parent_index: NodeIndex, child_index: NodeIndex, confirmed_only: bool) -> bool {
        let child_parenting = self.nodes[child_index].parenting();
        let parent_parenting = self.nodes[parent_index].parenting();
        child_parenting.parent() == Some(parent_index) && parent_parenting.children(confirmed_only).contains(&child_index)
        // cycle breaking
        // child_parenting.my_depth() > parent_parenting.my_depth()
    }

    fn send_messages(&mut self) {
        for (i, n) in self.nodes.iter().enumerate() {
            for m in n.send_messages() {
                assert!(
                    m.destination != i,
                    "Node {i} tried to send message to itself"
                );
                self.messages[m.destination].push(ReceivedMessage {
                    source: i,
                    message: m.message,
                });
            }
        }
    }

    fn receive_messages(&mut self, rng: &mut impl Rng) {
        for i in 0..self.messages.len() {
            let n = &mut self.nodes[i];
            let messages = std::mem::take(&mut self.messages[i]);
            n.receive_messages(messages, &self.conf, i, rng);
        }
    }

    fn subtree(&self, node_index: NodeIndex, confirmed_only: bool) -> Vec<NodeIndex> {
        let parenting_root = self.nodes[node_index].parenting();
        let root_depth = parenting_root.my_depth();
        let mut result = vec![node_index];
        parenting_root
            .children(confirmed_only)
            .iter()
            .filter(|i| {
                let child_parenting = self.nodes[**i].parenting();
                child_parenting.parent() == Some(node_index) &&
                // cycle breaking
                child_parenting.my_depth() > root_depth } )
            .for_each(|i| result.extend(self.subtree(*i, confirmed_only)));
        result
    }
}

fn no_messages(conf: &Configuration) -> MessageRouting {
    vec![ReceivedMessages::new(); conf.n]
}