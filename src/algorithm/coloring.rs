use crate::algorithm::RandomizableData;
use crate::messages::Message;
use crate::messages::ReceivedMessage;
use crate::tree_color::TreeColor;
use crate::Configuration;
use crate::Rng;

#[derive(Debug)]
pub struct ColoringData {
    color: TreeColor,
    prev_color: TreeColor,
    subtree_colored: bool,

    recoloring_rounds: usize,
}

impl RandomizableData for ColoringData {
    fn new_random(conf: &crate::Configuration, rng: &mut impl rand::Rng) -> Self {
        Self {
            color: TreeColor::new_random(conf, rng),
            prev_color: TreeColor::new_random(conf, rng),
            subtree_colored: false,
            recoloring_rounds: 0,
        }
    }

    fn reset(&mut self, conf: &crate::Configuration, rng: &mut impl rand::Rng) {
        *self = Self::new_random(conf, rng);
    }
}

impl ColoringData {
    pub fn new(color: TreeColor) -> Self {
        Self {
            color,
            prev_color: color,
            subtree_colored: false,
            recoloring_rounds: 0,
        }
    }

    pub fn curr_color(&self) -> TreeColor {
        self.color
    }

    pub fn is_valid_color(&self, color: TreeColor) -> bool {
        let r = color == self.color || color == self.prev_color;
        if !r {
            println!(
                "invalid color {:?} with color: {:?} prev_color: {:?}",
                color, self.color, self.prev_color
            )
        }
        r
    }

    pub fn subtree_color(&self) -> Option<TreeColor> {
        if self.subtree_colored {
            Some(self.color)
        } else {
            None
        }
    }

    pub fn recolor(&mut self, color: TreeColor) {
        if color == self.color {
            return;
        }
        self.recoloring_rounds = 0;
        self.prev_color = self.color;
        self.color = color;
        self.subtree_colored = false;
    }

    pub fn process_confirmations(
        &mut self,
        messages: &[ReceivedMessage],
        is_root: bool,
        conf: &Configuration,
        rng: &mut impl Rng,
    ) {
        self.recoloring_rounds += 1;
        self.subtree_colored = itertools::all(messages.iter(), |m| {
            if let Message::Confirmation(Some(color)) = m.message {
                color == self.color
            } else {
                false
            }
        });
        if is_root && self.subtree_colored {
            println!(
                "Done recoloring tree to {:?}, guessing depth {}",
                self.color,
                self.recoloring_rounds / 2
            );
            let new_color = TreeColor::new_random(conf, rng);
            self.recolor(if new_color != self.color {
                new_color
            } else {
                TreeColor::of(new_color.value + 1)
            });
            println!("Starting new color {:?}", self.color);
        }
    }
}
