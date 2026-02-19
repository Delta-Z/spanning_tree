use super::coloring::ColoringData;
use crate::algorithm::RandomizableData;
use crate::messages::Message::{Confirmation, Request};
use crate::messages::ReceivedMessage;
use crate::tree_id::TreeId;
use crate::tree_info::TreeInfo;
use crate::Configuration;
use crate::Rng;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum RoundType {
    ExchangeConfirmations,
    ExchangeRequests,
}

#[derive(Debug, Clone)]
pub struct TimersData {
    round_type: RoundType,
    reset_countdown: usize,
    convergence_timer: usize,
}

impl RoundType {
    fn next(&self) -> Self {
        match self {
            RoundType::ExchangeConfirmations => RoundType::ExchangeRequests,
            RoundType::ExchangeRequests => RoundType::ExchangeConfirmations,
        }
    }
}

impl RandomizableData for TimersData {
    fn new_random(conf: &Configuration, rng: &mut impl Rng) -> Self {
        Self {
            round_type: if rng.random_bool(1.0) {
                // TODO
                RoundType::ExchangeConfirmations
            } else {
                RoundType::ExchangeRequests
            },
            reset_countdown: rng.random_range(0..=conf.reset_countdown_max()),
            convergence_timer: rng.random_range(0..=conf.num_rounds_to_convergence()),
        }
    }

    fn reset(&mut self, conf: &Configuration, _: &mut impl rand::Rng) {
        *self = Self::new(conf);
    }
}

impl Display for TimersData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_counting_down_to_reset() {
            writeln!(f, "reset in {}", self.reset_countdown)?;
        }
        if !self.should_have_converged() {
            writeln!(f, "conv in {}", self.convergence_timer)?;
        }
        Ok(())
    }
}

impl TimersData {
    pub fn new(conf: &Configuration) -> Self {
        Self {
            round_type: RoundType::ExchangeRequests,
            reset_countdown: 0,
            convergence_timer: conf.num_rounds_to_convergence(),
        }
    }

    pub fn is_counting_down_to_reset(&self) -> bool {
        self.reset_countdown > 0
    }

    pub fn get_round_type(&self) -> RoundType {
        self.round_type.clone()
    }

    pub fn should_reset(&self) -> bool {
        self.reset_countdown == 1
    }

    pub fn advance_time(&mut self) {
        if self.convergence_timer > 0 {
            self.convergence_timer -= 1;
        }
        if self.reset_countdown > 1 {
            self.reset_countdown -= 1;
        }
        self.round_type = self.round_type.next();
    }

    pub fn reset_countdown(&self) -> usize {
        self.reset_countdown
    }

    pub fn update_for_configuration(&mut self, conf: &Configuration) {
        self.reset_countdown = self.reset_countdown.min(conf.reset_countdown_max());
        self.convergence_timer = self.convergence_timer.min(conf.num_rounds_to_convergence());
    }

    fn start_reset(&mut self, conf: &Configuration) -> bool {
        if !self.is_counting_down_to_reset() {
            self.reset_countdown = conf.reset_countdown_max();
            true
        } else {
            false
        }
    }

    fn merge(&mut self, tree_info: &TreeInfo) {
        self.reset_countdown = self.reset_countdown.max(tree_info.reset_countdown);
    }

    fn should_have_converged(&self) -> bool {
        self.convergence_timer == 0
    }
}

pub fn process_messages(
    messages: &[ReceivedMessage],
    timers: &mut TimersData,
    my_tree_id: &TreeId,
    coloring: &ColoringData,
    is_root: bool,
    conf: &Configuration,
) {
    for m in messages {
        match m.message {
            Request(ref request_tree_info) => {
                if timers.round_type == RoundType::ExchangeConfirmations && timers.start_reset(conf)
                {
                    println!("{}: unexpected request - resetting", my_tree_id);
                    return;
                }
                if is_root
                    && timers.should_have_converged()
                    && request_tree_info.tree_id == *my_tree_id
                    && !coloring.is_valid_color(request_tree_info.color)
                    && timers.start_reset(conf)
                {
                    println!("{}: unexpected color - resetting", my_tree_id);
                    return;
                }
                timers.merge(request_tree_info);
            }
            Confirmation(_) => {
                if timers.round_type == RoundType::ExchangeRequests && timers.start_reset(conf) {
                    println!("{}: unexpected confirmation - resetting", my_tree_id);
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let conf = Configuration::new(10, 2, 1.0, 10);
        let mut td = TimersData::new(&conf);
        assert_eq!(td.to_string(), "conv in 10\n");
        td.advance_time();
        td.start_reset(&conf);
        assert_eq!(td.to_string(), "reset in 10\nconvin 9\n");
        td.advance_time();
        assert_eq!(td.to_string(), "reset in 9\nconv in 8\n");
    }
}
