use super::board_builder::BoardBuilder;
use super::config::BACKGROUND_COLOR;
use super::config::RESTART_WINDOW_BG_COLOR;
use super::config::RESTART_WINDOW_HEIGHT;
use super::config::RESTART_WINDOW_WIDTH;
use super::elements::restart_window::RestartWindow;
use super::events::Event;
use super::mouse::MouseHandler;
use super::presenter::Presenter;
use crate::core::board::*;
use crate::core::entities::Player;
use crate::core::game::*;
use crate::core::state::*;
use macroquad::prelude::*;

pub type ShapeId = usize;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum UiStatus {
    Idle,
    Busy,
}

pub struct Frontend {
    width: f32,
    height: f32,
    pixel_width: u32,
    pixel_height: u32,
    w_margin: f32,
    h_margin: f32,
    radius: f32,
    presenter: Presenter,
    builder: BoardBuilder,
    mouse_handler: MouseHandler,
    pub ui_status: UiStatus,
    font: Font,
    update_request: bool,
}

impl Frontend {
    pub fn new(
        board: &Board,
        font: Font,
        pixel_width: u32,
        pixel_height: u32,
        w_margin: f32,
        h_margin: f32,
    ) -> Self {
        let radius = board.get_radius();
        let width = 2. * radius + w_margin;
        let height = 2. * radius + h_margin;

        Frontend {
            width,
            height,
            pixel_width,
            pixel_height,
            w_margin,
            h_margin,
            radius,
            presenter: Presenter::new(),
            mouse_handler: MouseHandler::new(width, height, pixel_width, pixel_height),
            ui_status: UiStatus::Idle,
            font,
            update_request: true,
            builder: BoardBuilder::new(radius, font),
        }
    }

    fn set_camera(&self) {
        set_camera(&Camera2D {
            zoom: vec2(1. / self.width * 2., 1. / self.height * 2.),
            target: vec2(0., 0.),
            ..Default::default()
        });
    }

    fn update_window_size(&mut self) {
        let pixel_width = screen_width();
        let pixel_height = screen_height();

        if (pixel_width - self.pixel_width as f32).abs() < 0.5
            || (pixel_height - self.pixel_height as f32).abs() < 0.5
        {
            return;
        }

        let min_width = 2. * self.radius + self.w_margin;
        let min_height = 2. * self.radius + self.h_margin;

        let min_ratio = min_width / min_height;
        let current_ratio = pixel_width / pixel_height / min_ratio;

        if pixel_width > min_ratio * pixel_height {
            self.height = min_height;
            self.width = current_ratio * min_width;
        }
        if pixel_width <= min_ratio * pixel_height {
            self.width = min_width;
            self.height = 1. / current_ratio * min_height;
        }

        self.pixel_height = pixel_height.round() as u32;
        self.pixel_width = pixel_width.round() as u32;

        self.mouse_handler =
            MouseHandler::new(self.width, self.height, self.pixel_width, self.pixel_height);
    }

    fn update_if_idle(&mut self, state: &State) {
        if self.ui_status == UiStatus::Idle && self.update_request {
            self.presenter.clear_all();
            let mut interactive = true;

            if let Some(winner) = state.won_by() {
                interactive = false;

                let win_text = match winner {
                    Player::White => "Congrats, you won!",
                    Player::Black => "You lost ...",
                };
                let score_text = format!("{} - {}", state.points_white, state.points_black);

                self.presenter.add_element(Box::new(RestartWindow::new(
                    vec2(-4.0, -2.0),
                    8.0,
                    4.0,
                    RESTART_WINDOW_BG_COLOR,
                    win_text,
                    &score_text,
                    self.font,
                    100,
                )));
            }
            self.builder
                .create_board_from_state(state, &mut self.presenter, interactive);
            self.update_request = false;
        }
    }

    fn schedule_mouse_events(&mut self, state: &State) {
        self.mouse_handler.update();
        let mouse_event = self.mouse_handler.has_message(Some(&state.legal_moves()));
        self.presenter.schedule_event(Event::Mouse(mouse_event));
    }

    fn handle_ui_actions(&mut self) -> UiAction {
        let mut ui_actions = self.presenter.get_actions();

        let is_animating = ui_actions
            .iter()
            .filter(|&a| a == &UiAction::AnimationInProgress)
            .count()
            != 0;

        if is_animating {
            self.ui_status = UiStatus::Busy;
            return UiAction::Busy;
        } else {
            self.ui_status = UiStatus::Idle;
        }

        ui_actions.retain(|a| match a {
            UiAction::ActionAtCoord(_) | UiAction::Undo | UiAction::Restart => true,
            _ => false,
        });
        if ui_actions.len() > 0 {
            println!("UIActions: {:?}", ui_actions);
        }

        ui_actions.pop().unwrap_or(UiAction::Idle)
    }
}

impl View for Frontend {
    fn request_update(&mut self) {
        self.update_request = true;
    }

    fn tick(&mut self, state: &State) -> UiAction {
        self.update_window_size();
        self.set_camera();
        self.update_if_idle(state);
        self.schedule_mouse_events(state);
        self.presenter.handle_events();

        clear_background(BACKGROUND_COLOR);
        self.presenter.render();

        self.handle_ui_actions()
    }
}
