use std::{collections::HashMap, fmt};

use crate::pragmata_pro_input::{code_to_codepoint, Segment};
use winit::keyboard::{Key, ModifiersState};

mod azerty;
pub use azerty::Azerty;

mod qwerty;
pub use qwerty::Qwerty;

#[path = "bépo.rs"]
mod bépo;
pub use bépo::Bépo;

pub const NONE: ModifiersState = ModifiersState::empty();
pub const ALT: ModifiersState = ModifiersState::ALT;
pub const SHIFT: ModifiersState = ModifiersState::SHIFT;
pub const CTRL: ModifiersState = ModifiersState::CONTROL;

type InputMap = HashMap<(ModifiersState, Key), Vec<Action>>;

#[derive(Clone)]
pub struct BoxMode;

#[derive(Clone)]
pub struct TextMode;

#[derive(Clone)]
pub struct ColorMode;

#[derive(Clone)]
pub struct ExtraMode {
    pub buffer: Vec<char>,
}

impl ExtraMode {
    pub fn buffer_to_actions(&mut self, map: &dyn KeyMap) -> Vec<Action> {
        let b = std::mem::take(&mut self.buffer);
        if let Some(point) = code_to_codepoint(&b.into_iter().collect::<String>(), map) {
            vec![Action::DrawCharAtCursor(char::from_u32(point).unwrap())]
        } else {
            vec![]
        }
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
    Undo,
    Redo,
    Transition(InputMode),
}

pub enum InputModeIdentifier {
    Box,
    Text,
    Color,
    Extra,
}

#[derive(Clone)]
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
    pub fn identifier(&self) -> InputModeIdentifier {
        match self {
            InputMode::Box(_) => InputModeIdentifier::Box,
            InputMode::Text(_) => InputModeIdentifier::Text,
            InputMode::Color(_) => InputModeIdentifier::Color,
            InputMode::Extra(_) => InputModeIdentifier::Extra,
        }
    }
}

pub trait KeyMap {
    fn translate(
        &self,
        mode: InputModeIdentifier,
        modifiers: ModifiersState,
        key: Key,
    ) -> Option<&'static [Action]>;

    fn char_to_extra_mode_segment(&self, c: char) -> Option<Segment>;
    fn extra_mode_segment_dot(&self, c: char) -> bool;
}
