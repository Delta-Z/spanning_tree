use crate::algorithm::RandomizableData;
use crate::messages::ReceivedMessage;
use crate::node::Node;
use crate::tree_color::TreeColor;
use crate::tree_id::TreeId;
use crate::Configuration;
use rand::Rng;
use std::fmt;

#[derive(Debug)]
pub struct Graph {
    conf: Configuration,
    nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct Tree<'a> {
    nodes: Vec<usize>,
    graph: &'a Graph,
}

impl Tree<'_> {
    pub fn new<'a>(graph: &'a Graph, nodes: Vec<usize>) -> Tree<'a> {
        assert!(!nodes.is_empty(), "Tree cannot be empty!");
        Tree::<'a> { nodes, graph }
    }

    pub fn root(&self) -> usize {
        self.nodes[0]
    }

    pub fn nodes(&self) -> &[usize] {
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

    pub fn graph(&self) -> &Graph {
        self.graph
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
    type Item = usize;
    type IntoIter = std::vec::IntoIter<usize>;

    fn into_iter(self) -> <Self as std::iter::IntoIterator>::IntoIter {
        self.nodes.into_iter()
    }
}

impl Graph {
    pub fn new_random(conf: Configuration, rng: &mut impl Rng) -> Self {
        let mut g = Graph {
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
        Graph { conf, nodes }
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
        if downsized {
            self.nodes
                .iter_mut()
                .for_each(|n| n.update_for_configuration(&self.conf));
        }
    }

    pub fn trees<'a>(&'a self) -> Vec<Tree<'a>> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if n.parenting().is_root() {
                    Some(Tree::new(self, self.subtree(i)))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn execute_round(&mut self, rng: &mut impl Rng) {
        let mut message_routing = vec![Vec::<ReceivedMessage>::new(); self.nodes.len()];
        for (i, n) in self.nodes.iter().enumerate() {
            for m in n.send_messages() {
                assert!(
                    m.destination != i,
                    "Node {i} tried to send message to itself"
                );
                message_routing[m.destination].push(ReceivedMessage {
                    source: i,
                    message: m.message,
                });
            }
        }
        for (i, messages) in message_routing.into_iter().enumerate() {
            let n = &mut self.nodes[i];
            n.receive_messages(messages, &self.conf, i, rng);
            n.advance_time(&self.conf, rng);
        }
    }

    fn subtree(&self, node_index: usize) -> Vec<usize> {
        let mut result = vec![node_index];
        self.nodes[node_index]
            .parenting()
            .confirmed_children()
            .iter()
            .for_each(|i| result.extend(self.subtree(*i)));
        result
    }
}
