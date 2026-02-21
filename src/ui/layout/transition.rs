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
        } else {
            self.transition = None;
        }
    }
}


#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::LayoutWithTransitions;
    use crate::{tree_color::TreeColor, tree_id::TreeId, ui::layout::*};

    fn make_layout(g: &Graph) -> Box<dyn GraphLayout> {
        graph_layout_for(g, ViewMode::Chord, RootPositions::Constant)
    }

    #[test]
    fn new_simple() {
        let g = Graph::new_test(vec![(TreeId::new_simple(1), TreeColor::of(1))], 1);
        let mut layout = LayoutWithTransitions::new(make_layout(&g));
        
        assert!(!layout.is_in_transition());
        layout.tick(Instant::now());
        assert!(layout.transition.is_none());
        assert!(!layout.is_in_transition());

        let start = Instant::now();
        layout.transition_to(make_layout(&g), Duration::from_secs(2));
        assert_eq!(layout.transition.as_ref().unwrap().1, 0.0);

        layout.tick(start + Duration::from_secs(1));
        let midpoint = layout.transition.as_ref().unwrap().1;
        assert!(midpoint > 0.0);
        assert!(midpoint < 1.0);
        assert!(layout.is_in_transition());

        layout.tick(start + Duration::from_secs(2));
        assert!(layout.transition.is_none());
        assert!(!layout.is_in_transition());
    }
}
