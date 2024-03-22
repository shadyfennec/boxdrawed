use std::{
    collections::HashMap,
    fmt,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
};

use lazy_static::lazy_static;
use minifb::{InputCallback, Key};

use crate::pragmata_pro_input::{code_to_codepoint, DummyCallback, ExtraInputBuffer, InputAction};

pub type Modifiers = u8;

pub const MODIFIER_NONE: u8 = 0b000;
pub const MODIFIER_SHIFT: u8 = 0b001;
pub const MODIFIER_CTRL: u8 = 0b010;
pub const MODIFIER_ALT: u8 = 0b100;

type InputMap = HashMap<(Modifiers, Key), Vec<Action>>;

lazy_static! {
    static ref BOX_MODE_CONTROLS: InputMap = {
        #[rustfmt::skip]
        let m: Vec<((Modifiers, Key), Vec<Action>)> = vec![
            // Block characters
            ((MODIFIER_NONE, Key::Space), vec![Action::DrawCharAtCursor(' ')]),
            ((MODIFIER_NONE, Key::Z), vec![Action::DrawCharAtCursor('│')]),
            ((MODIFIER_NONE, Key::X), vec![Action::DrawCharAtCursor('─')]),
            ((MODIFIER_NONE, Key::Q), vec![Action::DrawCharAtCursor('┌')]),
            ((MODIFIER_NONE, Key::W), vec![Action::DrawCharAtCursor('┐')]),
            ((MODIFIER_NONE, Key::A), vec![Action::DrawCharAtCursor('└')]),
            ((MODIFIER_NONE, Key::S), vec![Action::DrawCharAtCursor('┘')]),

            // Controls
            ((MODIFIER_NONE, Key::Left), vec![Action::CursorLeft]),
            ((MODIFIER_NONE, Key::Right), vec![Action::CursorRight]),
            ((MODIFIER_NONE, Key::Up), vec![Action::CursorUp]),
            ((MODIFIER_NONE, Key::Down), vec![Action::CursorDown]),
            ((MODIFIER_NONE, Key::Delete), vec![Action::DeleteAtCursor]),
            ((MODIFIER_NONE, Key::Backspace), vec![Action::CursorLeft, Action::DeleteAtCursor]),
            ((MODIFIER_NONE, Key::Minus), vec![Action::ReduceFontSize]),
            ((MODIFIER_NONE, Key::Equal), vec![Action::IncreaseFontSize]),

            // Transition
            ((MODIFIER_CTRL, Key::Key1), vec![Action::Transition(Box::new(BoxMode::transition))]),
            ((MODIFIER_CTRL, Key::Key2), vec![Action::Transition(Box::new(TextMode::transition))]),
            ((MODIFIER_CTRL, Key::Key3), vec![Action::Transition(Box::new(ColorMode::transition))]),
            ((MODIFIER_CTRL, Key::Key4), vec![Action::Transition(Box::new(ExtraMode::transition))]),
        ];

        m.into_iter().collect()
    };

    static ref TEXT_MODE_CONTROLS: InputMap = {
        #[rustfmt::skip]
        let m: Vec<((Modifiers, Key), Vec<Action>)> = vec![
            // Controls
            ((MODIFIER_NONE, Key::Left), vec![Action::CursorLeft]),
            ((MODIFIER_NONE, Key::Right), vec![Action::CursorRight]),
            ((MODIFIER_NONE, Key::Up), vec![Action::CursorUp]),
            ((MODIFIER_NONE, Key::Down), vec![Action::CursorDown]),
            ((MODIFIER_NONE, Key::Delete), vec![Action::DeleteAtCursor]),
            ((MODIFIER_NONE, Key::Backspace), vec![Action::CursorLeft, Action::DeleteAtCursor]),
            ((MODIFIER_NONE, Key::Minus), vec![Action::ReduceFontSize]),
            ((MODIFIER_NONE, Key::Equal), vec![Action::IncreaseFontSize]),

            // Transition
            ((MODIFIER_CTRL, Key::Key1), vec![Action::Transition(Box::new(BoxMode::transition))]),
            ((MODIFIER_CTRL, Key::Key2), vec![Action::Transition(Box::new(TextMode::transition))]),
            ((MODIFIER_CTRL, Key::Key3), vec![Action::Transition(Box::new(ColorMode::transition))]),
            ((MODIFIER_CTRL, Key::Key4), vec![Action::Transition(Box::new(ExtraMode::transition))]),
        ];

        m.into_iter().collect()
    };

    static ref COLOR_MODE_CONTROLS: InputMap = {
        #[rustfmt::skip]
        let m: Vec<((Modifiers, Key), Vec<Action>)> = vec![
            // Controls
            ((MODIFIER_NONE, Key::Left), vec![Action::CursorLeft]),
            ((MODIFIER_NONE, Key::Right), vec![Action::CursorRight]),
            ((MODIFIER_NONE, Key::Up), vec![Action::CursorUp]),
            ((MODIFIER_NONE, Key::Down), vec![Action::CursorDown]),
            ((MODIFIER_NONE, Key::Delete), vec![Action::DeleteAtCursor]),
            ((MODIFIER_NONE, Key::Backspace), vec![Action::CursorLeft, Action::DeleteAtCursor]),
            ((MODIFIER_NONE, Key::Minus), vec![Action::ReduceFontSize]),
            ((MODIFIER_NONE, Key::Equal), vec![Action::IncreaseFontSize]),

            // Transition
            ((MODIFIER_CTRL, Key::Key1), vec![Action::Transition(Box::new(BoxMode::transition))]),
            ((MODIFIER_CTRL, Key::Key2), vec![Action::Transition(Box::new(TextMode::transition))]),
            ((MODIFIER_CTRL, Key::Key3), vec![Action::Transition(Box::new(ColorMode::transition))]),
            ((MODIFIER_CTRL, Key::Key4), vec![Action::Transition(Box::new(ExtraMode::transition))]),
        ];

        m.into_iter().collect()
    };

    static ref EXTRA_MODE_CONTROLS: InputMap = {
        #[rustfmt::skip]
        let m: Vec<((Modifiers, Key), Vec<Action>)> = vec![
            // Controls
            ((MODIFIER_NONE, Key::Left), vec![Action::CursorLeft]),
            ((MODIFIER_NONE, Key::Right), vec![Action::CursorRight]),
            ((MODIFIER_NONE, Key::Up), vec![Action::CursorUp]),
            ((MODIFIER_NONE, Key::Down), vec![Action::CursorDown]),
            ((MODIFIER_NONE, Key::Delete), vec![Action::DeleteAtCursor]),
            // ((MODIFIER_NONE, Key::Backspace), vec![Action::CursorLeft, Action::DeleteAtCursor]),
            ((MODIFIER_NONE, Key::Minus), vec![Action::ReduceFontSize]),
            ((MODIFIER_NONE, Key::Equal), vec![Action::IncreaseFontSize]),

            // Transition
            ((MODIFIER_CTRL, Key::Key1), vec![Action::Transition(Box::new(BoxMode::transition))]),
            ((MODIFIER_CTRL, Key::Key2), vec![Action::Transition(Box::new(TextMode::transition))]),
            ((MODIFIER_CTRL, Key::Key3), vec![Action::Transition(Box::new(ColorMode::transition))]),
            ((MODIFIER_CTRL, Key::Key4), vec![Action::Transition(Box::new(ExtraMode::transition))]),
        ];

        m.into_iter().collect()
    };
}

pub struct BoxMode;

impl BoxMode {
    pub fn transition() -> (InputMode, Box<dyn InputCallback + Sync>) {
        (InputMode::Box(BoxMode), Box::new(DummyCallback))
    }
}

struct TextModeCallback {
    tx: Sender<char>,
}

impl InputCallback for TextModeCallback {
    fn add_char(&mut self, uni_char: u32) {
        if let Some(c) = char::from_u32(uni_char) {
            if !c.is_control() {
                let _ = self.tx.send(c);
            }
        }
    }
}

impl TextModeCallback {
    pub fn new() -> (Self, Receiver<char>) {
        let (tx, rx) = channel();
        (Self { tx }, rx)
    }
}

pub struct TextMode {
    rx: Receiver<char>,
}

impl TextMode {
    pub fn transition() -> (InputMode, Box<dyn InputCallback + Sync>) {
        let (callback, rx) = TextModeCallback::new();

        (InputMode::Text(TextMode { rx }), Box::new(callback))
    }
}

pub struct ColorMode;

impl ColorMode {
    pub fn transition() -> (InputMode, Box<dyn InputCallback + Sync>) {
        (InputMode::Color(ColorMode), Box::new(DummyCallback))
    }
}

pub struct ExtraMode {
    pub rx: Receiver<InputAction>,
    pub buffer: Vec<char>,
}

impl ExtraMode {
    pub fn transition() -> (InputMode, Box<dyn InputCallback + Sync>) {
        let (callback, rx) = ExtraInputBuffer::new();
        (
            InputMode::Extra(ExtraMode {
                rx,
                buffer: Vec::new(),
            }),
            Box::new(callback),
        )
    }
}

pub enum Action {
    CursorLeft,
    CursorRight,
    CursorUp,
    CursorDown,
    DrawCharAtCursor(char),
    DeleteAtCursor,
    ReduceFontSize,
    IncreaseFontSize,
    Transition(Box<dyn Fn() -> (InputMode, Box<dyn InputCallback + Sync>) + Sync>),
}

pub enum InputMode {
    Box(BoxMode),
    Text(TextMode),
    Color(ColorMode),
    Extra(ExtraMode),
}

impl fmt::Display for InputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputMode::Box(_) => write!(f, "Box"),
            InputMode::Text(_) => write!(f, "Text"),
            InputMode::Color(_) => write!(f, "Color"),
            InputMode::Extra(_) => write!(f, "Extra"),
        }
    }
}

impl InputMode {
    fn input_map(&self) -> &'static InputMap {
        match self {
            InputMode::Box(_) => &BOX_MODE_CONTROLS,
            InputMode::Text(_) => &TEXT_MODE_CONTROLS,
            InputMode::Color(_) => &COLOR_MODE_CONTROLS,
            InputMode::Extra(_) => &EXTRA_MODE_CONTROLS,
        }
    }

    pub fn process_callbacks(&mut self) -> Vec<Action> {
        match self {
            InputMode::Extra(ExtraMode { rx, buffer }) => match rx.try_recv() {
                Ok(c) => match c {
                    InputAction::Char(c) => {
                        if c.is_whitespace() {
                            let b = std::mem::take(buffer);
                            if let Some(point) =
                                code_to_codepoint(&b.into_iter().collect::<String>())
                            {
                                vec![Action::DrawCharAtCursor(char::from_u32(point).unwrap())]
                            } else {
                                vec![]
                            }
                        } else {
                            buffer.push(c);
                            vec![]
                        }
                    }
                    InputAction::Backspace => {
                        if buffer.is_empty() {
                            vec![Action::CursorLeft, Action::DeleteAtCursor]
                        } else {
                            buffer.pop();
                            vec![]
                        }
                    }
                },
                Err(TryRecvError::Disconnected) => {
                    panic!("extra input channel disconnected")
                }
                Err(TryRecvError::Empty) => {
                    vec![]
                }
            },
            InputMode::Text(TextMode { rx }) => {
                let mut v = Vec::new();
                while let Ok(c) = rx.try_recv() {
                    v.push(Action::DrawCharAtCursor(c));
                    v.push(Action::CursorRight)
                }
                v
            }
            _ => vec![],
        }
    }

    pub fn handle_keys(&mut self, keys: Vec<Key>, modifiers: Modifiers) -> Vec<&'static Action> {
        let map = self.input_map();

        keys.into_iter()
            .filter_map(|k| map.get(&(modifiers, k)))
            .flatten()
            .collect()
    }
}
