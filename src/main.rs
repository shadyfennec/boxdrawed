use std::{collections::VecDeque, path::PathBuf, time::Instant};

use clap::Parser;
use fontdue::{Font, FontSettings};
use minifb::{Key, KeyRepeat, Window, WindowOptions};

mod canvas;
use canvas::Canvas;

mod input_mode;
use input_mode::{Action, BoxMode, ExtraMode, InputMode};

mod pragmata_pro_input;
use text_area::Direction;

mod text_area;

use crate::input_mode::{Modifiers, MODIFIER_ALT, MODIFIER_CTRL, MODIFIER_NONE, MODIFIER_SHIFT};

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

struct App {
    window: Window,
    canvas: Canvas,
    input_mode: InputMode,
    frame_durations: VecDeque<u64>,
}

impl App {
    pub fn new(window: Window, font: Font, font_size: f32, width: usize, height: usize) -> Self {
        Self {
            window,
            canvas: Canvas::new(font, font_size, width, height),
            input_mode: InputMode::Box(BoxMode),
            frame_durations: VecDeque::with_capacity(64),
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
                Action::ReduceFontSize => {
                    if self.canvas.font_size() > 6.0 {
                        self.canvas.set_font_size(self.canvas.font_size() - 2.0);
                    }
                }
                Action::IncreaseFontSize => {
                    self.canvas.set_font_size(self.canvas.font_size() + 2.0);
                }
                Action::Undo => self.canvas.draw_area.undo(),
                Action::Redo => self.canvas.draw_area.redo(),
            }
        }
    }

    fn top_line(&mut self) {
        let frames = self.frame_durations.iter().sum::<u64>();
        let frames = if self.frame_durations.is_empty() {
            0
        } else {
            frames / self.frame_durations.len() as u64
        } as f64
            / 1_000_000_000.0;

        self.canvas.top_line.reset_cursor();
        self.canvas.top_line.clear();
        self.canvas.top_line.write_string_at_cursor(&format!(
            "[{:.2}fps] X = {}, Y = {}, mode = {}",
            1.0 / frames,
            self.canvas.draw_area.cursor_absolute_position().x,
            self.canvas.draw_area.cursor_absolute_position().y,
            self.input_mode,
        ));
    }

    fn bottom_line(&mut self) {
        self.canvas.bottom_line.reset_cursor();
        self.canvas.bottom_line.clear();

        if let InputMode::Extra(ExtraMode { rx: _, buffer }) = &self.input_mode {
            if !buffer.is_empty() {
                self.canvas.bottom_line.write_string_at_cursor(&format!(
                    "Char code: {}",
                    buffer.iter().copied().collect::<String>()
                ));
            }
        }
    }

    pub fn update(&mut self) {
        let start = Instant::now();
        self.handle_keys();

        self.top_line();
        self.bottom_line();

        self.canvas.render();

        self.window
            .update_with_buffer(
                self.canvas.get_buffer(),
                self.canvas.width(),
                self.canvas.height(),
            )
            .unwrap();

        self.frame_durations
            .push_back(start.elapsed().as_nanos() as u64);
        if self.frame_durations.len() > 64 {
            self.frame_durations.pop_front();
        }
    }
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the font to use
    font: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut window = Window::new("BoxDrawEd", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let font =
        Font::from_bytes(std::fs::read(args.font).unwrap(), FontSettings::default()).unwrap();
    let mut app = App::new(window, font, 36.0, WIDTH, HEIGHT);

    while app.is_open() {
        app.update()
    }
}
