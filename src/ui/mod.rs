use crate::graph::NodeIndex;
use crate::tree_id::TreeId;
use crate::ui::layout::RootPositions;
use graph_renderer::GraphRenderer;
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::time::Instant;
use iced::widget::container;
use iced::widget::row;
use iced::widget::slider;
use iced::widget::text_input;
use iced::widget::Float;
use iced::widget::Stack;
use iced::widget::{button, canvas, checkbox, column, space, text, Column};
use iced::window;
use iced::Element;
use iced::Fill;
use iced::Length;
use iced::Subscription;
use layout::ViewMode;
use messages::Message;
use messages::TreeIdEdit;
use std::str::FromStr;

mod graph_renderer;
pub mod layout;
mod messages;
pub mod timer;

enum GraphEditing {
    EditNode(NodeIndex, String),
}

#[derive(Default)]
pub struct App {
    gr: GraphRenderer,
    round_number: usize,
    graph_editing: Option<GraphEditing>,
}

const CONTAINER_PADDING_PX: f32 = 10.0;

impl App {
    pub fn run() -> iced::Result {
        iced::application::timed(Self::default, Self::update, Self::subscription, Self::view).run()
    }

    fn view(&self) -> Column<'_, Message> {
        let mut graphics = vec![container(canvas(&self.gr).width(Fill).height(Fill))
            .padding(CONTAINER_PADDING_PX)
            .into()];
        if let Some(graph_editing) = &self.graph_editing {
            match graph_editing {
                GraphEditing::EditNode(node_index, output) => {
                    graphics.push(self.node_editor(*node_index, output))
                }
            }
        }

        let graphics_container = Stack::from_vec(graphics).width(Fill).height(Fill);
        let num_trees = self.gr.num_trees();
        let mut status = format!("round {}, {} tree", self.round_number, num_trees).to_string();
        if num_trees > 1 {
            status += "s";
        }
        if let Some(reset_countdown) = self.gr.reset_countdown() {
            status += &format!(", resetting in {} rounds", reset_countdown).to_string();
        }
        let controls = container(row![
            text(format!("Graph size {} ", self.gr.graph_size())).align_y(Vertical::Center),
            slider(3.0..=30.0, self.gr.graph.nodes().len() as f32, |x: f32| {
                Message::ResizeGraph(x as usize)
            }),
            space::horizontal().width(CONTAINER_PADDING_PX),
            checkbox(self.gr.settings().view_mode == ViewMode::Forest)
                .label("show trees")
                .on_toggle(|v| {
                    Message::ViewMode(if v { ViewMode::Forest } else { ViewMode::Chord })
                }),
            space::horizontal().width(CONTAINER_PADDING_PX),
            checkbox(self.gr.settings().root_positions == RootPositions::Sorted)
                .label("sort by tree size")
                .on_toggle(|v| {
                    Message::RootPositions(if v {
                        RootPositions::Sorted
                    } else {
                        RootPositions::Constant
                    })
                }),
            space::horizontal().width(CONTAINER_PADDING_PX),
            checkbox(self.gr.settings().show_tentative_requests)
                .label("show tentative requests")
                .on_toggle(Message::ShowTentativeRequests),
            space::horizontal(),
            text(status).align_y(Vertical::Center),
            space::horizontal().width(CONTAINER_PADDING_PX),
            button("next").on_press(Message::NextRound),
        ])
        .padding(CONTAINER_PADDING_PX)
        .center_x(Fill);

        column![graphics_container, controls]
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.gr.is_animating() {
            window::frames().map(|_| Message::Animate)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, m: Message, now: Instant) {
        match m {
            Message::NextRound => {
                self.round_number += 1;
                println!("Round {}", self.round_number);
                self.graph_editing = None;
            }
            Message::EditNode(node_index, ref edit) => {
                let edit = match edit {
                    TreeIdEdit::Valid(tree_id) => tree_id.to_string(),
                    TreeIdEdit::Invalid(s) => s.to_string(),
                };
                self.graph_editing = Some(GraphEditing::EditNode(node_index, edit));
            }
            _ => {}
        }
        self.gr.tick(now);
        self.gr.apply_update(m);
    }

    fn node_editor(&self, node_index: NodeIndex, contents: &str) -> Element<'_, Message> {
        Float::new(
            text_input(&self.gr.graph.nodes()[node_index].to_string(), contents)
                .width(Length::Fixed(40.0))
                .on_input(move |s| {
                    Message::EditNode(
                        node_index,
                        TreeId::from_str(&s).map_or(TreeIdEdit::Invalid(s), TreeIdEdit::Valid),
                    )
                }),
        )
        .translate(move |bounds, _window_size| {
            let node_bounds = self.gr.node_bounds(node_index).unwrap();
            node_bounds.anchor(bounds.size(), Horizontal::Center, Vertical::Center)
                - bounds.position()
        })
        .into()
    }
}
