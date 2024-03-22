use fontdue::{Font, FontSettings};
use minifb::{Key, KeyRepeat, Window, WindowOptions};

mod canvas;

mod input_mode;
use input_mode::{Action, BoxMode, InputMode};

mod pragmata_pro_input;
use text_area::Direction;

mod text_area;

use crate::input_mode::{Modifiers, MODIFIER_ALT, MODIFIER_CTRL, MODIFIER_NONE, MODIFIER_SHIFT};

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

const FONT: &[u8] = include_bytes!("../resources/pp.ttf");

struct App {
    window: Window,
    canvas: canvas::Canvas,
    input_mode: InputMode,
}

impl App {
    pub fn new(window: Window, font: Font, font_size: f32, width: usize, height: usize) -> Self {
        Self {
            window,
            canvas: canvas::Canvas::new(font, font_size, width, height),
            input_mode: InputMode::Box(BoxMode),
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    fn get_modifiers(&self) -> Modifiers {
        let mut modifiers = MODIFIER_NONE;

        if self.window.is_key_down(Key::LeftShift) || self.window.is_key_down(Key::RightShift) {
            modifiers |= MODIFIER_SHIFT
        }

        if self.window.is_key_down(Key::LeftCtrl) || self.window.is_key_down(Key::RightCtrl) {
            modifiers |= MODIFIER_CTRL
        }

        if self.window.is_key_down(Key::LeftAlt) || self.window.is_key_down(Key::RightAlt) {
            modifiers |= MODIFIER_ALT
        }

        modifiers
    }

    fn handle_keys(&mut self) {
        let callbacks = self.input_mode.process_callbacks();

        for action in callbacks.iter().chain(self.input_mode.handle_keys(
            self.window.get_keys_pressed(KeyRepeat::Yes),
            self.get_modifiers(),
        )) {
            match action {
                Action::CursorLeft => self.canvas.draw_area.move_cursor(Direction::Left),
                Action::CursorRight => self.canvas.draw_area.move_cursor(Direction::Right),
                Action::CursorUp => self.canvas.draw_area.move_cursor(Direction::Up),
                Action::CursorDown => self.canvas.draw_area.move_cursor(Direction::Down),
                Action::DrawCharAtCursor(c) => self.canvas.draw_area.write_at_cursor(*c),
                Action::DeleteAtCursor => self.canvas.draw_area.erase_at_cursor(),
                Action::Transition(f) => {
                    let (mode, callback) = (f)();

                    self.window.set_input_callback(callback);
                    self.input_mode = mode;
                }
            }
        }
    }

    pub fn update(&mut self) {
        self.handle_keys();
        self.canvas.render();

        self.window
            .update_with_buffer(
                self.canvas.get_buffer(),
                self.canvas.width(),
                self.canvas.height(),
            )
            .unwrap()
    }
}

fn main() {
    let mut window = Window::new("BoxDrawEd", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let font = Font::from_bytes(FONT, FontSettings::default()).unwrap();
    let mut app = App::new(window, font, 36.0, WIDTH, HEIGHT);

    while app.is_open() {
        app.update()
    }
}
