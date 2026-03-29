use crate::algorithm::RandomizableData;
use crate::graph::NodeIndex;
use crate::messages::Message;
use crate::messages::ReceivedMessage;
use crate::messages::SentMessage;
use crate::random_peer_generator::PeerGenerator;
use crate::tree_id::TreeId;
use crate::tree_info::TreeInfo;
use crate::Configuration;
use crate::Rng;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct ParentingData {
    tree_id: TreeId,
    parent: Option<NodeIndex>,
    my_depth: usize,
    children: Vec<NodeIndex>,
    num_confirmed_children: usize,
}

struct ReceivedRequest {
    pub source: NodeIndex,
    pub tree_info: TreeInfo,
}

impl RandomizableData for ParentingData {
    fn new_random(conf: &Configuration, rng: &mut impl Rng) -> Self {
        let is_root = conf.draw_from_initial_root_probablity(rng);
        Self {
            tree_id: TreeId::new_random(conf.id_range, rng),
            parent: if is_root {
                None
            } else {
                Some(rng.random_range(0..conf.n))
            },
            my_depth: if is_root {
                0
            } else {
                rng.random_range(1..conf.n)
            },
            children: Vec::with_capacity(conf.n),
            num_confirmed_children: 0,
        }
    }

    fn reset(&mut self, conf: &Configuration, rng: &mut impl rand::Rng) {
        self.tree_id = TreeId::new(conf.epsilon, rng);
        self.parent = None;
        self.my_depth = 0;
        self.num_confirmed_children = 0;
    }
}

impl ParentingData {
    pub fn new_root(tree_id: TreeId) -> Self {
        Self {
            tree_id,
            parent: None,
            my_depth: 0,
            children: vec![],
            num_confirmed_children: 0,
        }
    }

    pub fn new_random_root(conf: &Configuration, rng: &mut impl Rng) -> Self {
        Self::new_root(TreeId::new(conf.epsilon, rng))
    }

    pub fn tree_id(&self) -> &TreeId {
        &self.tree_id
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn parent(&self) -> Option<NodeIndex> {
        self.parent
    }

    pub fn my_depth(&self) -> usize {
        self.my_depth
    }

    pub fn confirmed_children(&self) -> &[NodeIndex] {
        &self.children[..self.num_confirmed_children]
    }

    pub fn tentative_children(&self) -> &[NodeIndex] {
        &self.children[self.num_confirmed_children..]
    }

    pub fn update_for_configuration(&mut self, conf: &Configuration) {
        self.children.retain(|i| *i < conf.n);
        if self.children.len() > conf.d {
            self.children.truncate(conf.d);
        }
        self.num_confirmed_children = 0;
        self.parent = self.parent.filter(|i| *i < conf.n);
    }

    pub fn set_tree_id(&mut self, value: TreeId) {
        self.tree_id = value
    }

    fn adopt_parent(&mut self, parent_request: &ReceivedRequest) {
        self.parent = Some(parent_request.source);
        self.tree_id = parent_request.tree_info.tree_id;
        self.my_depth = parent_request.tree_info.depth + 1;
    }

    fn become_root(&mut self) {
        self.parent = None;
        self.my_depth = 0;
    }

    fn regenerate_children(
        &mut self,
        conf: &Configuration,
        confirmed_children: Vec<NodeIndex>,
        mut pg: impl PeerGenerator,
    ) {
        self.children
            .retain(|child_index| confirmed_children.contains(child_index));
        self.num_confirmed_children = self.children.len();

        let mut excludes = self.children.clone();
        self.parent.iter().for_each(|p| excludes.push(*p));
        pg.exclude(excludes);

        while self.children.len() < conf.d {
            let Some(candidate) = pg.generate_peer() else {
                break;
            };
            assert!(
                candidate < conf.n
                    && self.parent != Some(candidate)
                    && !self.children.contains(&candidate),
                "Invalid candidate id {candidate}!"
            );
            self.children.push(candidate);
        }
    }

    fn elect_parent(&self, requests: Vec<ReceivedRequest>) -> Option<ReceivedRequest> {
        requests
            .into_iter()
            .max_by(|t1, t2| {
                t1.tree_info.cmp(&t2.tree_info).then(
                    // Nit: parent stability for nicer vis, non-essential for the algorithm.
                    if self.parent == Some(t1.source) {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    },
                )
            })
            .filter(
                |candidate| match candidate.tree_info.tree_id.cmp(&self.tree_id) {
                    Ordering::Less => false,
                    Ordering::Equal => candidate.tree_info.depth < self.my_depth,
                    Ordering::Greater => true,
                },
            )
    }

    pub fn process_requests(&mut self, messages: &[ReceivedMessage]) -> Option<TreeInfo> {
        let requests = messages
            .iter()
            .filter_map(|m| {
                if let Message::Request(tree_info) = &m.message {
                    Some(ReceivedRequest {
                        source: m.source,
                        tree_info: tree_info.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();
        if let Some(parent) = self.elect_parent(requests) {
            if self.parent != Some(parent.source) {
                println!("Joining [{}] {}", parent.source, parent.tree_info.tree_id);
            }
            self.adopt_parent(&parent);
            Some(parent.tree_info)
        } else {
            if !self.is_root() {
                println!("Becoming root {}", self.tree_id);
            }
            self.become_root();
            None
        }
    }

    pub fn process_confirmations(
        &mut self,
        messages: &[ReceivedMessage],
        conf: &Configuration,
        pg: impl PeerGenerator,
    ) {
        self.regenerate_children(
            conf,
            messages
                .iter()
                .filter_map(|m| {
                    if let Message::Confirmation(_) = m.message {
                        Some(m.source)
                    } else {
                        None
                    }
                })
                .collect(),
            pg,
        );
    }

    pub fn send_requests(&self, my_tree_info: TreeInfo) -> Vec<SentMessage> {
        self.children
            .iter()
            .map(|c| SentMessage {
                destination: *c,
                message: Message::Request(my_tree_info.clone()),
            })
            .collect()
    }
}
