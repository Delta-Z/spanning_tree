use std::time::{Duration, Instant};

use super::layout::{self, GraphLayout, NodeCenterPoint, RootPositions, ViewMode};
use super::messages::Message;
use crate::algorithm::timers::RoundType;
use crate::graph::{Graph, NodeIndex};
use crate::messages::Message as NodeMessage;
use crate::node::Node;
use crate::tree_color::TreeColor;
use crate::tree_id::TreeId;
use crate::ui::canvas::Text;
use crate::ui::layout::transition::LayoutWithTransitions;
use crate::Configuration;
use iced::mouse::{self, Button};
use iced::widget::canvas::{self, path, Cache, Frame, Geometry, LineDash, Path, Stroke};
use iced::{alignment, Color, Event, Point, Rectangle, Renderer, Size, Theme, Vector};
use rand::rngs::ThreadRng;

#[derive(Copy, Clone, Debug)]
pub struct Settings {
    pub view_mode: ViewMode,
    pub root_positions: RootPositions,
    pub show_tentative_requests: bool,
}

pub struct GraphRenderer {
    pub graph: Graph,
    settings: Settings,
    layout: LayoutWithTransitions,
    render_cache: Cache,
    rng: ThreadRng,
}

const NODE_COLOR: Color = Color::from_rgb8(200, 200, 200);
const COUNTDOWN_TO_RESET_COLOR: Color = Color::from_rgb8(255, 0, 0);
const NODE_REQUEST_BORDER_RATIO: f32 = 0.1;
const REQUEST_ARC_WIDTH: f32 = 3.0;

impl iced::widget::canvas::Program<Message> for GraphRenderer {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        vec![self
            .render_cache
            .draw(renderer, bounds.size(), |frame| self.render(frame))]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    let node_radius = self.layout.node_radius(bounds.size());
                    for (i, node_center) in self
                        .layout
                        .arrange_nodes(bounds.size())
                        .into_iter()
                        .enumerate()
                    {
                        if cursor_position.distance(node_center) < node_radius {
                            println!("click {} -> {} [{:?}]", i, node_center, bounds.size());
                            return Some(
                                canvas::Action::publish(Message::EditNode(i)).and_capture(),
                            );
                        }
                    }
                    Some(canvas::Action::publish(Message::NextRound).and_capture())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Chord,
            root_positions: RootPositions::Constant,
            show_tentative_requests: true,
        }
    }
}

impl Default for GraphRenderer {
    fn default() -> Self {
        let settings = Settings::default();
        let graph = Graph::new_test(
            vec![
                (TreeId::new_simple(1), TreeColor::of(1)),
                (TreeId::new_simple(2), TreeColor::of(2)),
                (TreeId::new_simple(3), TreeColor::of(3)),
            ],
            Configuration::default().d,
        );
        let layout = LayoutWithTransitions::new(layout::graph_layout_for(
            &graph,
            settings.view_mode,
            settings.root_positions,
        ));
        Self {
            graph,
            settings,
            layout,
            rng: rand::rng(),
            render_cache: Cache::default(),
        }
    }
}

impl GraphRenderer {
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn graph_size(&self) -> usize {
        self.graph.nodes().len()
    }

    pub fn num_trees(&self) -> usize {
        self.graph
            .nodes()
            .iter()
            .filter(|n| n.parenting().is_root())
            .count()
    }

    pub fn reset_countdown(&self) -> Option<usize> {
        self.graph
            .nodes()
            .iter()
            .filter_map(|n| {
                let countdown = n.timers().reset_countdown();
                if countdown > 0 {
                    Some(countdown)
                } else {
                    None
                }
            })
            .max()
    }

    fn render(&self, frame: &mut Frame<Renderer>) {
        let node_positions = self.layout.arrange_nodes(frame.size());
        let node_colors: Vec<Color> = self
            .graph
            .nodes()
            .iter()
            .map(|n| self.tree_color(n.color()))
            .collect();
        let node_radius = self.layout.node_radius(frame.size());
        for (i, pos) in node_positions.iter().enumerate() {
            self.draw_node(
                frame,
                self.graph.nodes().get(i).unwrap(),
                node_radius,
                *pos,
                node_colors[i],
            );
            for incoming_message in &self.graph.in_flight_messages()[i] {
                self.draw_message(
                    frame,
                    node_positions[incoming_message.source],
                    *pos,
                    &incoming_message.message,
                    node_colors[incoming_message.source],
                );
            }
        }
    }

    fn draw_node(
        &self,
        frame: &mut Frame<Renderer>,
        node: &Node,
        node_radius: f32,
        center: NodeCenterPoint,
        color: Color,
    ) {
        let node_circle = Path::circle(center, node_radius);
        frame.fill(&node_circle, NODE_COLOR);

        let text_size = layout::text_size(&self.layout, frame.size());
        let extra_text = node.timers().to_string();
        frame.fill_text(Text {
            content: node.parenting().tree_id().to_string(),
            size: text_size,
            position: center,
            align_x: alignment::Alignment::Center.into(),
            align_y: alignment::Vertical::Bottom,
            ..canvas::Text::default()
        });
        if !extra_text.is_empty() {
            frame.fill_text(Text {
                content: extra_text,
                color: COUNTDOWN_TO_RESET_COLOR,
                size: text_size * 0.8,
                position: center,
                align_x: alignment::Alignment::Center.into(),
                align_y: alignment::Vertical::Top,
                ..canvas::Text::default()
            });
        }

        if node.timers().get_round_type() == RoundType::ExchangeRequests {
            frame.stroke(
                &node_circle,
                Stroke::default()
                    .with_color(color)
                    .with_width((node_radius * NODE_REQUEST_BORDER_RATIO).min(5.0)),
            );
        }
    }

    fn tree_color(&self, tree_color: TreeColor) -> Color {
        const MAX_RGB: u32 = (1u32 << 24) - 1;
        let color_position_in_range =
            tree_color.value as f64 / self.graph.configuration().max_color() as f64;
        let bytes = ((MAX_RGB as f64 * color_position_in_range) as u32 ^ 0xa5285a).to_le_bytes();
        Color::from_rgb8(bytes[0], bytes[1], bytes[2])
    }

    fn draw_message(
        &self,
        frame: &mut Frame<Renderer>,
        from: NodeCenterPoint,
        to: NodeCenterPoint,
        message: &NodeMessage,
        source_color: Color,
    ) {
        let stroke = match message {
            NodeMessage::Request(ti) => {
                if !self.settings.show_tentative_requests {
                    return;
                }
                stroke(self.tree_color(ti.color), true)
            }
            NodeMessage::Confirmation(Some(color)) => stroke(self.tree_color(*color), false),
            NodeMessage::Confirmation(None) => stroke(source_color.scale_alpha(0.5), false),
        };
        frame.stroke(
            &request_arc(from, to, self.layout.node_radius(frame.size()), None),
            stroke,
        )
    }

    pub fn is_animating(&self) -> bool {
        self.layout.is_in_transition()
    }

    pub fn tick(&mut self, now: Instant) {
        self.layout.tick(now);
    }

    pub fn apply_update(&mut self, m: Message) {
        match m {
            Message::ResizeGraph(new_size) => self.graph.resize(new_size, &mut self.rng),
            Message::NextRound => self.graph.execute_round(&mut self.rng),
            Message::RootPositions(new_value) => self.settings.root_positions = new_value,
            Message::ShowTentativeRequests(new_value) => {
                self.settings.show_tentative_requests = new_value
            }
            Message::ViewMode(new_value) => self.settings.view_mode = new_value,
            Message::Animate | Message::EditNode(_) => {}
        }
        match m {
            Message::ResizeGraph(_) => {
                self.layout = LayoutWithTransitions::new(self.new_graph_layout())
            }
            Message::RootPositions(_) | Message::ViewMode(_) | Message::NextRound => {
                self.layout.transition_to(
                    self.new_graph_layout(),
                    Instant::now(),
                    Duration::from_secs(2),
                )
            }
            _ => {}
        }
        self.render_cache.clear();
    }

    pub fn node_bounds(&self, node_index: NodeIndex, viewport_size: Size) -> Option<Rectangle> {
        let node_center = *self.layout.arrange_nodes(viewport_size).get(node_index)?;
        Some(Rectangle::new(node_center, Size::ZERO).expand(self.layout.node_radius(viewport_size)))
    }

    fn new_graph_layout(&self) -> Box<dyn GraphLayout> {
        layout::graph_layout_for(
            &self.graph,
            self.settings.view_mode,
            self.settings.root_positions,
        )
    }
}

fn request_arc(
    from: NodeCenterPoint,
    to: NodeCenterPoint,
    node_radius: f32,
    bend_fn: Option<fn(Point) -> Point>,
) -> Path {
    let direction = Vector::new(to.x - from.x, to.y - from.y) / from.distance(to);
    let node_offset = direction * node_radius;
    let parent_side = from + node_offset;
    let mut path = path::Builder::new();
    path.move_to(parent_side);
    path.circle(parent_side, 10.0);
    if let Some(bend_fn) = bend_fn {
        let midpoint =
            Point::new(0.0, 0.0) + (Vector::new(from.x, from.y) + Vector::new(to.x, to.y)) * 0.5;
        let control_point = bend_fn(midpoint);
        path.quadratic_curve_to(control_point, to - direction * node_radius);
    } else {
        path.line_to(to - node_offset);
    }
    path.build()
}

fn stroke(color: Color, dashed: bool) -> Stroke<'static> {
    Stroke {
        style: color.into(),
        width: REQUEST_ARC_WIDTH,
        line_dash: if dashed {
            LineDash {
                segments: &[REQUEST_ARC_WIDTH * 2.0, REQUEST_ARC_WIDTH * 2.0],
                ..LineDash::default()
            }
        } else {
            LineDash::default()
        },
        ..Stroke::default()
    }
}
