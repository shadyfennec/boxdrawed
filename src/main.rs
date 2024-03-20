use std::collections::HashMap;

use fontdue::{layout::GlyphRasterConfig, Font, FontSettings, Metrics};
use minifb::{Key, KeyRepeat, Window, WindowOptions};

mod input_mode;
use input_mode::{Action, BoxMode, ColorMode, ExtraMode, InputMode, TextMode};

mod pragmata_pro_input;
use pragmata_pro_input::{code_to_codepoint, DummyCallback, ExtraInputBuffer};

use crate::input_mode::{Modifiers, MODIFIER_ALT, MODIFIER_CTRL, MODIFIER_NONE, MODIFIER_SHIFT};

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

const FONT: &[u8] = include_bytes!("../resources/pp.ttf");

struct Canvas {
    window: Window,
    font: Font,
    pixel_buffer: Vec<u32>,
    text_buffer: Vec<Option<char>>,
    font_size: f32,
    lines: usize,
    cols: usize,
    cursor: usize,
    atlas: HashMap<GlyphRasterConfig, (Metrics, Vec<u8>)>,
    insert_mode: bool,
    input_mode: InputMode,
}

impl Canvas {
    pub fn new(window: Window, font: Font, font_size: f32) -> Self {
        if font_size.floor() as u32 % 2 != 0 {
            panic!("font_size must be divisible by 2")
        }

        let lines = (HEIGHT / font_size.floor() as usize) - 1;
        let cols = WIDTH / (font_size.floor() as usize / 2);

        Self {
            window,
            font,
            pixel_buffer: vec![0; WIDTH * HEIGHT],
            text_buffer: vec![None; (lines + 1) * cols],
            font_size,
            lines,
            cols,
            cursor: 0,
            atlas: HashMap::new(),
            insert_mode: false,
            input_mode: InputMode::Box(BoxMode),
        }
    }

    fn config_of(&self, c: char) -> GlyphRasterConfig {
        GlyphRasterConfig {
            glyph_index: self.font.lookup_glyph_index(c),
            px: self.font_size,
            font_hash: self.font.file_hash(),
        }
    }

    fn rasterize(&mut self, c: char) -> &(Metrics, Vec<u8>) {
        self.atlas
            .entry(self.config_of(c))
            .or_insert_with(|| self.font.rasterize(c, self.font_size))
    }

    fn clear_modeline(&mut self) {
        self.text_buffer[self.lines * self.cols..self.lines * self.cols + self.cols]
            .copy_from_slice(&vec![None; self.cols]);
    }

    fn draw(&mut self) {
        let mut text_buffer = std::mem::take(&mut self.text_buffer);
        let mut pixel_buffer = std::mem::take(&mut self.pixel_buffer);
        let lines = self.lines;
        let cols = self.cols;
        let font_size = self.font_size as usize;
        let cursor = self.cursor;
        let (cursor_x, cursor_y) = (
            (cursor % cols) * (font_size / 2),
            (cursor / cols) * font_size,
        );

        if let InputMode::Extra(ExtraMode { rx: _, buffer }) = &self.input_mode {
            let chars = buffer
                .iter()
                .copied()
                .take(self.cols)
                .map(Some)
                .chain(std::iter::repeat(None).take(cols.saturating_sub(buffer.len())))
                .collect::<Vec<_>>();

            text_buffer[lines * cols..(lines * cols) + chars.len()].copy_from_slice(&chars);
        }

        for (x, y, c) in text_buffer
            .iter()
            .enumerate()
            .filter_map(|(idx, c)| c.map(|c| (idx % cols, idx / cols, c)))
        {
            let pixel_x = x * (font_size / 2);
            let pixel_y = y * font_size;

            let (metrics, bitmap) = self.rasterize(c);

            if !bitmap.is_empty() {
                let skip = (pixel_y as isize) + (font_size as isize - metrics.height as isize)
                    - metrics.ymin as isize;
                let take = if skip < 0 {
                    metrics.height - skip.unsigned_abs()
                } else {
                    metrics.height
                };

                for (dest, src) in pixel_buffer
                    .chunks_mut(WIDTH)
                    .skip(skip.max(0) as usize)
                    .take(take)
                    .zip(bitmap.chunks(metrics.width))
                {
                    let x = pixel_x.saturating_add_signed(metrics.xmin as isize);
                    for (src, dest) in src
                        .iter()
                        .map(|b| [0, *b, *b, *b])
                        .zip(dest[x..x + metrics.width].iter_mut())
                    {
                        let mut d = dest.to_be_bytes();

                        if d != src {
                            d[1] = d[1].saturating_add(src[1]);
                            d[2] = d[2].saturating_add(src[2]);
                            d[3] = d[3].saturating_add(src[3]);

                            *dest = u32::from_be_bytes(d);
                        }
                    }
                }
            }
        }

        for l in pixel_buffer
            .chunks_mut(WIDTH)
            .skip(cursor_y + if self.insert_mode { font_size / 2 } else { 0 })
            .take(font_size - if self.insert_mode { font_size / 2 } else { 0 })
        {
            for v in &mut l[cursor_x..cursor_x + (font_size / 2)] {
                let [_, a, b, c] = v.to_be_bytes();
                *v = u32::from_be_bytes([0, !a, !b, !c]);
            }
        }

        self.pixel_buffer = pixel_buffer;
        self.text_buffer = text_buffer;
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

    fn change_input_mode(&mut self, mut new: InputMode) {
        std::mem::swap(&mut new, &mut self.input_mode);

        if let InputMode::Extra(ExtraMode { rx: _, buffer: _ }) = new {
            self.text_buffer[self.lines * self.cols..self.lines * self.cols + self.cols]
                .copy_from_slice(&vec![None; self.cols]);
            self.window.set_input_callback(Box::new(DummyCallback));
        }
    }

    fn handle_keys(&mut self) {
        let callbacks = self.input_mode.process_callbacks();

        for action in callbacks.iter().chain(self.input_mode.handle_keys(
            self.window.get_keys_pressed(KeyRepeat::Yes),
            self.get_modifiers(),
        )) {
            match action {
                Action::CursorLeft => {
                    if self.cursor == 0 {
                        self.cursor = self.lines * self.cols - 1
                    } else {
                        self.cursor -= 1
                    }
                }
                Action::CursorRight => self.cursor = (self.cursor + 1) % (self.lines * self.cols),
                Action::CursorUp => {
                    if self.cursor < self.cols {
                        self.cursor = (self.lines * self.cols) - (self.cols - self.cursor)
                    } else {
                        self.cursor -= self.cols
                    }
                }
                Action::CursorDown => {
                    self.cursor = (self.cursor + self.cols) % (self.cols * self.lines)
                }
                Action::DrawCharAtCursor(c) => self.push_char(*c),
                Action::DeleteAtCursor => self.text_buffer[self.cursor] = None,
                Action::ToggleInsertMode => self.insert_mode = !self.insert_mode,
                Action::Transition(f) => {
                    let (mode, callback) = (f)();
                    self.clear_modeline();
                    self.window.set_input_callback(callback);
                    self.input_mode = mode;
                }
            }
        }
    }

    pub fn update(&mut self) {
        self.pixel_buffer.copy_from_slice(&[0; WIDTH * HEIGHT]);
        self.handle_keys();
        self.draw();

        self.window
            .update_with_buffer(&self.pixel_buffer, WIDTH, HEIGHT)
            .unwrap()
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    pub fn push_char(&mut self, c: char) {
        self.text_buffer[self.cursor] = Some(c);

        if !self.insert_mode {
            self.cursor = (self.cursor + 1) % (self.cols * self.lines);
        }
    }
}

fn main() {
    let mut window = Window::new("BoxDrawEd", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let font = Font::from_bytes(FONT, FontSettings::default()).unwrap();

    let mut canvas = Canvas::new(window, font, 36.0);

    while canvas.is_open() {
        canvas.update()
    }
}
