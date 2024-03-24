use std::{
    collections::{HashSet, VecDeque},
    num::NonZeroU32,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

use clap::Parser;
use fontdue::{Font, FontSettings};
use keymap::{Action, Azerty, BoxMode, ExtraMode, InputMode, KeyMap};
use softbuffer::{Context, Surface};
use winit::{
    event::{ElementState, Event, KeyEvent, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowBuilder},
};

mod canvas;
use canvas::Canvas;

mod keymap;

mod pragmata_pro_input;
use text_area::Direction;

mod text_area;

struct App {
    window: Rc<Window>,
    key_map: Box<dyn KeyMap>,
    canvas: Canvas,
    input_mode: InputMode,
    frame_durations: VecDeque<u64>,
    keys: HashSet<Key>,
    modifiers: ModifiersState,
}

impl App {
    pub fn new(window: Rc<Window>, font: Font, font_size: f32, key_map: Box<dyn KeyMap>) -> Self {
        let context = Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();

        let size = window.inner_size();
        let (width, height) = (size.width as usize, size.height as usize);

        Self {
            window,
            key_map,
            canvas: Canvas::new(font, font_size, surface, width, height),
            input_mode: InputMode::Box(BoxMode),
            frame_durations: VecDeque::with_capacity(64),
            keys: HashSet::new(),
            modifiers: ModifiersState::empty(),
        }
    }

    /*
    fn handle_keys(&mut self) {
        self.keys = self.window.get_keys();
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
    */

    fn handle_action(&mut self, action: &Action) {
        match action {
            Action::CursorLeft => self.canvas.draw_area.move_cursor(Direction::Left),
            Action::CursorRight => self.canvas.draw_area.move_cursor(Direction::Right),
            Action::CursorUp => self.canvas.draw_area.move_cursor(Direction::Up),
            Action::CursorDown => self.canvas.draw_area.move_cursor(Direction::Down),
            Action::DrawCharAtCursor(c) => self.canvas.draw_area.write_at_cursor(*c),
            Action::DeleteAtCursor => self.canvas.draw_area.erase_at_cursor(),
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
            Action::Transition(mode) => self.input_mode = mode.clone(),
        }
    }

    pub fn handle_raw_key(&mut self, key: Key, _modifiers: ModifiersState) -> Vec<Action> {
        match &mut self.input_mode {
            InputMode::Box(_) | InputMode::Color(_) => vec![],
            InputMode::Text(_) => {
                if let Key::Character(s) = key {
                    s.chars()
                        .flat_map(|c| [Action::DrawCharAtCursor(c), Action::CursorRight])
                        .collect()
                } else {
                    vec![]
                }
            }
            InputMode::Extra(e) => match key {
                Key::Character(c) if c.chars().all(|c| c.is_whitespace()) => {
                    e.buffer_to_actions(&*self.key_map)
                }
                Key::Character(c) => {
                    e.buffer.extend(c.chars());
                    vec![]
                }
                Key::Named(NamedKey::Space | NamedKey::Enter) => {
                    e.buffer_to_actions(&*self.key_map)
                }
                Key::Named(NamedKey::Backspace) => {
                    if e.buffer.is_empty() {
                        vec![Action::CursorLeft, Action::DeleteAtCursor]
                    } else {
                        e.buffer.pop();
                        vec![]
                    }
                }
                _ => vec![],
            },
        }
    }

    fn run(mut self, event_loop: EventLoop<()>) {
        event_loop
            .run(move |event, elwt| {
                let start = Instant::now();

                elwt.set_control_flow(ControlFlow::wait_duration(Duration::from_millis(5)));

                self.top_line();
                self.bottom_line();

                match event {
                    Event::NewEvents(StartCause::ResumeTimeReached {
                        start: _,
                        requested_resume: _,
                    }) => {
                        self.window.request_redraw();
                    }
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::RedrawRequested,
                    } if window_id == self.window.id() => {
                        if let (Some(width), Some(height)) = {
                            let size = self.window.inner_size();
                            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                        } {
                            // Resize surface if needed
                            self.canvas.resize(width, height);

                            self.canvas.render();
                        }
                    }
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::Resized(size),
                    } if window_id == self.window.id() => {
                        if let (Some(width), Some(height)) =
                            { (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) }
                        {
                            self.canvas.resize(width, height);

                            self.canvas.render();
                        }
                    }
                    Event::WindowEvent {
                        event:
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        logical_key: Key::Named(NamedKey::Escape),
                                        ..
                                    },
                                ..
                            },
                        window_id,
                    } if window_id == self.window.id() => {
                        elwt.exit();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::KeyboardInput { event, .. },
                        window_id,
                    } if window_id == self.window.id() => match event.state {
                        ElementState::Pressed => {
                            if let Some(actions) = self.key_map.translate(
                                self.input_mode.identifier(),
                                self.modifiers,
                                event.logical_key.clone(),
                            ) {
                                self.keys.insert(event.logical_key);
                                for action in actions {
                                    self.handle_action(action)
                                }
                            } else {
                                for action in
                                    &self.handle_raw_key(event.logical_key, self.modifiers)
                                {
                                    self.handle_action(action)
                                }
                            }
                        }
                        ElementState::Released => {
                            self.keys.remove(&event.logical_key);
                        }
                    },
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::ModifiersChanged(modifiers),
                    } if window_id == self.window.id() => {
                        self.modifiers = modifiers.state();
                    }
                    _ => {}
                }

                self.frame_durations
                    .push_back(start.elapsed().as_nanos() as u64);
                if self.frame_durations.len() > 64 {
                    self.frame_durations.pop_front();
                }
            })
            .unwrap();
    }

    fn top_line(&mut self) {
        /*
        let frames = self.frame_durations.iter().sum::<u64>();
        let frames = if self.frame_durations.is_empty() {
            0
        } else {
            frames / self.frame_durations.len() as u64
        } as f64
            / 1_000_000_000.0;

        */
        self.canvas.top_line.reset_cursor();
        self.canvas.top_line.clear();
        self.canvas.top_line.write_string_at_cursor(&format!(
            "X = {}, Y = {}, mode = {}, keys = [{}]",
            //1.0 / frames,
            self.canvas.draw_area.cursor_absolute_position().x,
            self.canvas.draw_area.cursor_absolute_position().y,
            self.input_mode,
            self.keys
                .iter()
                .map(|k| format!("{k:?}"))
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }

    fn bottom_line(&mut self) {
        self.canvas.bottom_line.reset_cursor();
        self.canvas.bottom_line.clear();

        if let InputMode::Extra(ExtraMode { buffer }) = &self.input_mode {
            if !buffer.is_empty() {
                self.canvas.bottom_line.write_string_at_cursor(&format!(
                    "Char code: {}",
                    buffer.iter().copied().collect::<String>()
                ));
            }
        }
    }

    /*
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
    */
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the font to use
    font: PathBuf,
}

fn main() {
    let args = Args::parse();

    let event_loop = EventLoop::new().unwrap();
    let window = Rc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let font =
        Font::from_bytes(std::fs::read(args.font).unwrap(), FontSettings::default()).unwrap();

    let app = App::new(window, font, 24.0, Box::new(Azerty));

    app.run(event_loop)
}
