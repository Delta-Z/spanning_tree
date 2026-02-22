use crate::ui::timer::Timer;
use std::time::Duration;
use std::time::Instant;
use super::{GraphLayout, NodeCenterPoint, Size};
use iced::animation::Interpolable;
use itertools::Itertools;

pub struct LayoutWithTransitions {
    layout: Box<dyn GraphLayout>,
    prev_layout: Option<Box<dyn GraphLayout>>,
    transition: Timer,
}

impl GraphLayout for LayoutWithTransitions {
    fn node_radius(&self, viewport_size: Size) -> f32 {
        let new_radius = self.layout.node_radius(viewport_size);
        if let Some(prev_layout) = &self.prev_layout && self.transition.in_progress() {
            let old_radius = prev_layout.node_radius(viewport_size);
            old_radius.interpolated(new_radius, self.transition.elapsed_ratio())
        } else {
            new_radius
        }
    }

    fn arrange_nodes(&self, viewport_size: Size) -> Vec<NodeCenterPoint> {
        let new_positions = self.layout.arrange_nodes(viewport_size);
        if let Some(prev_layout) = &self.prev_layout && self.transition.in_progress() {
            let old_positions = prev_layout.arrange_nodes(viewport_size);
            let progress = self.transition.elapsed_ratio();
            old_positions.into_iter()
                .zip_eq(new_positions)
                .map(|(old, new)| {
                    NodeCenterPoint::new(old.x.interpolated(new.x, progress), old.y.interpolated(new.y, progress))
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
            transition: Timer::new_elapsed(),
        }
    }

    pub fn transition_to(&mut self, layout: Box<dyn GraphLayout>, start: Instant, duration: Duration) {
        self.prev_layout = Some(std::mem::replace(&mut self.layout, layout));
        self.transition = Timer::new(start, duration);
    }

    pub fn is_in_transition(&self) -> bool {
        self.transition.in_progress()
    }

    pub fn tick(&mut self, now: Instant) {
        self.transition.tick(now);
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
        assert!(!layout.is_in_transition());

        let start = Instant::now();
        layout.transition_to(make_layout(&g), start, Duration::from_secs(2));
        assert!(layout.is_in_transition());
        assert_eq!(layout.transition.elapsed_ratio(), 0.0);

        layout.tick(start + Duration::from_secs(1));
        assert!(layout.is_in_transition());
        assert_eq!(layout.transition.elapsed_ratio(), 0.5);

        layout.tick(start + Duration::from_secs(2));
        assert!(!layout.is_in_transition());
        assert_eq!(layout.transition.elapsed_ratio(), 1.0);
    }
}
