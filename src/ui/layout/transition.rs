use std::time::Duration;
use std::time::Instant;
use super::{GraphLayout, NodeCenterPoint, Size};
use iced::{Animation, animation::Interpolable};
use itertools::Itertools;

pub struct LayoutWithTransitions {
    layout: Box<dyn GraphLayout>,
    prev_layout: Option<Box<dyn GraphLayout>>,
    transition: Option<(Animation<bool>, f32)>,
}

impl GraphLayout for LayoutWithTransitions {
    fn node_radius(&self, viewport_size: Size) -> f32 {
        let new_radius = self.layout.node_radius(viewport_size);
        if let Some(prev_layout) = &self.prev_layout && let Some((_, transition_point)) = self.transition {
            let old_radius = prev_layout.node_radius(viewport_size);
            old_radius.interpolated(new_radius, transition_point)
        } else {
            new_radius
        }
    }

    fn arrange_nodes(&self, viewport_size: Size) -> Vec<NodeCenterPoint> {
        let new_positions = self.layout.arrange_nodes(viewport_size);
        if let Some(prev_layout) = &self.prev_layout && let Some((_, transition_point)) = self.transition {
            let old_positions = prev_layout.arrange_nodes(viewport_size);
            old_positions.into_iter()
                .zip_eq(new_positions)
                .map(|(old, new)| {
                    NodeCenterPoint::new(old.x.interpolated(new.x, transition_point), old.y.interpolated(new.y, transition_point))
                })
                .collect()
        } else {
            new_positions
        }
    }
}

impl LayoutWithTransitions {
    pub fn new(layout: Box<dyn GraphLayout>) -> Self {
        Self {
            layout,
            prev_layout: None,
            transition: None,
        }
    }

    pub fn transition_to(&mut self, layout: Box<dyn GraphLayout>, duration: Duration) {
        self.prev_layout = Some(std::mem::replace(&mut self.layout, layout));
        self.transition = Some((Animation::new(false).duration(duration).easing(iced::animation::Easing::EaseInOutCubic), 0.0));
    }

    pub fn is_in_transition(&self) -> bool {
        self.transition.is_some()
    }

    pub fn tick(&mut self, now: Instant) {
        let Some((animation, transition_point)) = self.transition.as_mut() else { return };
        if animation.is_animating(now) {
            *transition_point = animation.interpolate(0.0, 1.0, now);
        } else { self.transition = None  }
    }
}
