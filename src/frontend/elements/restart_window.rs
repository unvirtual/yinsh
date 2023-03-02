use macroquad::prelude::*;

use crate::{
    core::coord::{HexCoord, Point},
    core::game::UiAction,
    frontend::{
        config::{
            BUTTON_BORDER_COLOR, BUTTON_BORDER_WIDTH, BUTTON_DEFAULT_COLOR, BUTTON_FONT_SIZE,
            BUTTON_HOVER_COLOR, BUTTON_TEXT_COLOR, RESTART_WINDOW_SCORE_COLOR,
            RESTART_WINDOW_SCORE_FONTSIZE, RESTART_WINDOW_STATUS_COLOR,
            RESTART_WINDOW_STATUS_FONTSIZE,
        },
        element::Element,
        events::{Event, Message},
        mouse::mouse_leave_enter_event,
    },
};

pub struct RestartWindow {
    pos: Vec2,
    width: f32,
    height: f32,
    color: Color,
    font: Font,
    font_size: f32,
    status_text: String,
    score_text: String,
    button: Button,
    z_value: i32,
}

impl RestartWindow {
    pub fn new(
        pos: Vec2,
        width: f32,
        height: f32,
        color: Color,
        status_text: &str,
        score_text: &str,
        font: Font,
        z_value: i32,
    ) -> Self {
        let font_size = 1.;

        let button_pos = vec2(0., -1.);
        let button = Button::new(button_pos, 1., 0.25, font);

        Self {
            pos,
            width,
            height,
            color,
            font,
            font_size,
            status_text: status_text.to_owned(),
            score_text: score_text.to_owned(),
            button,
            z_value,
        }
    }

    fn contains(&self, pos: Point) -> bool {
        self.button.contains(pos)
    }

    fn set_status_text(&mut self, text: &str) {
        self.status_text = text.to_owned();
    }
}

impl Element for RestartWindow {
    fn render(&self) {
        draw_rectangle(
            self.pos.x + 0.2,
            self.pos.y - 0.2,
            self.width,
            self.height,
            DARKGRAY,
        );
        draw_rectangle(self.pos.x, self.pos.y, self.width, self.height, self.color);
        draw_text_centered(
            &self.status_text,
            vec2(0., 1.),
            self.font,
            RESTART_WINDOW_STATUS_FONTSIZE,
            RESTART_WINDOW_STATUS_COLOR,
        );
        draw_text_centered(
            &self.score_text,
            vec2(0., 0.),
            self.font,
            RESTART_WINDOW_SCORE_FONTSIZE,
            RESTART_WINDOW_SCORE_COLOR,
        );
        self.button.draw();
    }

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        match message {
            Message::MouseClicked(_) => Some(UiAction::Restart),
            Message::MouseEntered => {
                self.button.color = self.button.hover_color;
                None
            }
            Message::MouseLeft => {
                self.button.color = self.button.default_color;
                None
            }
            _ => None,
        }
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        match event {
            Event::Mouse(mouse_event) => {
                if mouse_event.left_clicked && self.contains(mouse_event.pos) {
                    res.push(Message::MouseClicked(
                        HexCoord::closest_coord_to_point(&mouse_event.pos).0,
                    ));
                }
                if let Some(e) = mouse_leave_enter_event(mouse_event, |pt| self.contains(*pt)) {
                    res.push(e);
                    return res;
                };
            }

            _ => (),
        }
        res
    }

    fn z_value(&self) -> i32 {
        self.z_value
    }
}

fn draw_text_centered(text: &str, center_pos: Vec2, font: Font, font_size: f32, color: Color) {
    let (font_size, font_scale, font_aspect) = camera_font_scale(font_size);
    let text_params = TextParams {
        font_size,
        font_scale: -font_scale,
        font_scale_aspect: -font_aspect,
        color,
        font,
        ..Default::default()
    };

    let center = get_text_center(text, Some(font), font_size, font_scale, 0.);
    draw_text_ex(
        text,
        -center.x + center_pos.x,
        center.y + center_pos.y,
        text_params,
    );
}

struct Button {
    pos: Vec2,
    width: f32,
    height: f32,
    color: Color,
    hover_color: Color,
    default_color: Color,
    label_text: String,
    label_text_params: TextParams,
    label_pos: Vec2,
    font: Font,
}

impl Button {
    fn new(center_pos: Vec2, left_right_margin: f32, top_bottom_margin: f32, font: Font) -> Self {
        let label_text = "PLAY AGAIN".to_owned();
        let (font_size, font_scale, font_aspect) = camera_font_scale(BUTTON_FONT_SIZE);

        let label_text_params = TextParams {
            font,
            font_size,
            font_scale: -font_scale,
            font_scale_aspect: -font_aspect,
            color: BUTTON_TEXT_COLOR,
            ..Default::default()
        };

        let label_center = get_text_center(&label_text, Some(font), font_size, font_scale, 0.);
        let label_dims = measure_text(&label_text, Some(font), font_size, font_scale);
        let (width, height) = (
            label_dims.width + left_right_margin,
            label_dims.height + top_bottom_margin,
        );
        let pos = center_pos - vec2(0.5 * width, 0.5 * height);

        let label_pos = center_pos - label_center - vec2(0., label_dims.offset_y);

        Self {
            pos,
            width,
            height,
            color: BUTTON_DEFAULT_COLOR,
            default_color: BUTTON_DEFAULT_COLOR,
            hover_color: BUTTON_HOVER_COLOR,
            label_text,
            label_pos,
            label_text_params,
            font,
        }
    }

    fn contains(&self, pos: Point) -> bool {
        pos.0 > self.pos.x
            && pos.0 < self.pos.x + self.width
            && pos.1 > self.pos.y
            && pos.1 < self.pos.y + self.height
    }

    fn draw(&self) {
        draw_rectangle(self.pos.x, self.pos.y, self.width, self.height, self.color);
        draw_rectangle_lines(
            self.pos.x,
            self.pos.y,
            self.width,
            self.height,
            BUTTON_BORDER_WIDTH,
            BUTTON_BORDER_COLOR,
        );
        draw_text_ex(
            &self.label_text,
            self.label_pos.x,
            self.label_pos.y,
            self.label_text_params,
        );
    }
}
