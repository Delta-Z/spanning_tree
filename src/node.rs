use crate::algorithm::coloring::ColoringData;
use crate::algorithm::parenting::ParentingData;
use crate::algorithm::timers;
use crate::algorithm::timers::RoundType;
use crate::algorithm::timers::TimersData;
use crate::algorithm::RandomizableData;
use crate::graph::NodeIndex;
use crate::messages::Message;
use crate::messages::ReceivedMessage;
use crate::messages::SentMessage;
use crate::random_peer_generator::RandomPeerGenerator;
use crate::tree_color::TreeColor;
use crate::tree_id::TreeId;
use crate::tree_info::TreeInfo;
use crate::Configuration;
use crate::Rng;
use std::fmt;

#[derive(Debug)]
pub struct Node {
    parenting: ParentingData,
    coloring: ColoringData,
    timers: TimersData,
}

impl RandomizableData for Node {
    fn new_random(conf: &Configuration, rng: &mut impl Rng) -> Self {
        Self {
            coloring: ColoringData::new_random(conf, rng),
            parenting: ParentingData::new_random(conf, rng),
            timers: TimersData::new_random(conf, rng),
        }
    }
    fn reset(&mut self, conf: &Configuration, rng: &mut impl rand::Rng) {
        self.parenting.reset(conf, rng);
        self.coloring.reset(conf, rng);
        self.timers.reset(conf, rng);
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.parenting.tree_id())
    }
}

impl Node {
    pub fn new_root(conf: &Configuration, tree_id: TreeId, color: TreeColor) -> Self {
        Self {
            coloring: ColoringData::new(color),
            parenting: ParentingData::new_root(tree_id),
            timers: TimersData::new(conf),
        }
    }

    pub fn new_random_root(conf: &Configuration, rng: &mut impl Rng) -> Self {
        Self {
            coloring: ColoringData::new_random(conf, rng),
            parenting: ParentingData::new_random_root(conf, rng),
            timers: TimersData::new(conf),
        }
    }

    pub fn parenting(&self) -> &ParentingData {
        &self.parenting
    }

    pub fn timers(&self) -> &TimersData {
        &self.timers
    }

    pub fn color(&self) -> TreeColor {
        self.coloring.curr_color()
    }

    pub fn advance_time(&mut self, conf: &Configuration, rng: &mut impl Rng) {
        if !self.timers.advance_time() {
            self.reset(conf, rng);
        }
    }

    pub fn receive_messages(
        &mut self,
        messages: Vec<ReceivedMessage>,
        conf: &Configuration,
        self_id: NodeIndex,
        rng: &mut impl Rng,
    ) {
        timers::process_messages(
            &messages,
            &mut self.timers,
            self.parenting.tree_id(),
            &self.coloring,
            self.parenting.is_root(),
            conf,
        );
        match self.timers.get_round_type() {
            RoundType::ExchangeRequests => {
                if let Some(parent) = self.parenting.process_requests(&messages) {
                    self.coloring.recolor(parent.color);
                }
            }
            RoundType::ExchangeConfirmations => {
                self.parenting.process_confirmations(
                    &messages,
                    conf,
                    RandomPeerGenerator::new(conf.n, vec![self_id], rng),
                );
                self.coloring
                    .process_confirmations(&messages, self.parenting.is_root(), conf, rng);
            }
        }
    }

    pub fn send_messages(&self) -> Vec<SentMessage> {
        match self.timers.get_round_type() {
            RoundType::ExchangeRequests => self.parenting.send_requests(TreeInfo {
                tree_id: *self.parenting.tree_id(),
                depth: self.parenting.my_depth(),
                color: self.coloring.curr_color(),
                reset_countdown: self.timers.reset_countdown(),
            }),
            RoundType::ExchangeConfirmations => self.send_confirmations(),
        }
    }

    fn send_confirmations(&self) -> Vec<SentMessage> {
        if self.parenting.is_root() {
            vec![]
        } else {
            vec![SentMessage {
                destination: self.parenting.parent().unwrap(),
                message: Message::Confirmation(self.coloring.subtree_color()),
            }]
        }
    }

    pub fn update_for_configuration(&mut self, conf: &Configuration) {
        self.parenting.update_for_configuration(conf);
        self.timers.update_for_configuration(conf);
    }

    pub fn set_tree_id(&mut self, value: TreeId) {
        self.parenting.set_tree_id(value)
    }
}
