use lazy_static::lazy_static;
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::{
    keymap::{BoxMode, ColorMode, ExtraMode, InputMode, TextMode},
    pragmata_pro_input::Segment,
};

use super::{Action, InputMap, InputModeIdentifier, KeyMap, CTRL, NONE};

lazy_static! {
    static ref BOX: InputMap = {
        #[rustfmt::skip]
        let m: Vec<((ModifiersState, Key), Vec<Action>)> = vec![
            // Block characters
            ((NONE, Key::Named(NamedKey::Space)), vec![Action::DrawCharAtCursor(' ')]),
            ((NONE, Key::Character(" ".into())), vec![Action::DrawCharAtCursor(' ')]),
            ((NONE, Key::Character("à".into())), vec![Action::DrawCharAtCursor('│')]),
            ((NONE, Key::Character("y".into())), vec![Action::DrawCharAtCursor('─')]),
            ((NONE, Key::Character("b".into())), vec![Action::DrawCharAtCursor('┌')]),
            ((NONE, Key::Character("é".into())), vec![Action::DrawCharAtCursor('┐')]),
            ((NONE, Key::Character("a".into())), vec![Action::DrawCharAtCursor('└')]),
            ((NONE, Key::Character("u".into())), vec![Action::DrawCharAtCursor('┘')]),
            ((NONE, Key::Character("p".into())), vec![Action::DrawCharAtCursor('┴')]),
            ((NONE, Key::Character("o".into())), vec![Action::DrawCharAtCursor('├')]),
            ((NONE, Key::Character("è".into())), vec![Action::DrawCharAtCursor('┬')]),
            ((NONE, Key::Character("!".into())), vec![Action::DrawCharAtCursor('┤')]),
            ((NONE, Key::Character("v".into())), vec![Action::DrawCharAtCursor('┼')]),

            // Controls
            ((NONE, Key::Named(NamedKey::ArrowLeft)), vec![Action::CursorLeft]),
            ((NONE, Key::Named(NamedKey::ArrowRight)), vec![Action::CursorRight]),
            ((NONE, Key::Named(NamedKey::ArrowUp)), vec![Action::CursorUp]),
            ((NONE, Key::Named(NamedKey::ArrowDown)), vec![Action::CursorDown]),
            ((NONE, Key::Named(NamedKey::Delete)), vec![Action::DeleteAtCursor]),
            ((NONE, Key::Named(NamedKey::Backspace)), vec![Action::CursorLeft, Action::DeleteAtCursor]),
            ((NONE, Key::Character("=".into())), vec![Action::ReduceFontSize]),
            ((NONE, Key::Character("%".into())), vec![Action::IncreaseFontSize]),
            ((CTRL, Key::Character("z".into())), vec![Action::Undo]),
            ((CTRL, Key::Character("y".into())), vec![Action::Redo]),

            // Transition
            ((CTRL, Key::Character("1".into())), vec![Action::Transition(InputMode::Box(BoxMode))]),
            ((CTRL, Key::Character("2".into())), vec![Action::Transition(InputMode::Text(TextMode))]),
            ((CTRL, Key::Character("3".into())), vec![Action::Transition(InputMode::Color(ColorMode))]),
            ((CTRL, Key::Character("4".into())), vec![Action::Transition(InputMode::Extra(ExtraMode { buffer: Vec::new()}))]),
        ];

        m.into_iter().collect()
    };

    static ref TEXT: InputMap = {
        #[rustfmt::skip]
        let m: Vec<((ModifiersState, Key), Vec<Action>)> = vec![
            // Controls
            ((NONE, Key::Named(NamedKey::ArrowLeft)), vec![Action::CursorLeft]),
            ((NONE, Key::Named(NamedKey::ArrowRight)), vec![Action::CursorRight]),
            ((NONE, Key::Named(NamedKey::ArrowUp)), vec![Action::CursorUp]),
            ((NONE, Key::Named(NamedKey::ArrowDown)), vec![Action::CursorDown]),
            ((NONE, Key::Named(NamedKey::Delete)), vec![Action::DeleteAtCursor]),
            ((NONE, Key::Named(NamedKey::Backspace)), vec![Action::CursorLeft, Action::DeleteAtCursor]),
            ((CTRL, Key::Character("z".into())), vec![Action::Undo]),
            ((CTRL, Key::Character("y".into())), vec![Action::Redo]),

            // Transition
            ((CTRL, Key::Character("1".into())), vec![Action::Transition(InputMode::Box(BoxMode))]),
            ((CTRL, Key::Character("2".into())), vec![Action::Transition(InputMode::Text(TextMode))]),
            ((CTRL, Key::Character("3".into())), vec![Action::Transition(InputMode::Color(ColorMode))]),
            ((CTRL, Key::Character("4".into())), vec![Action::Transition(InputMode::Extra(ExtraMode { buffer: Vec::new()}))]),
        ];

        m.into_iter().collect()
    };

    static ref COLOR: InputMap = {
        #[rustfmt::skip]
        let m: Vec<((ModifiersState, Key), Vec<Action>)> = vec![
            // Controls
            ((NONE, Key::Named(NamedKey::ArrowLeft)), vec![Action::CursorLeft]),
            ((NONE, Key::Named(NamedKey::ArrowRight)), vec![Action::CursorRight]),
            ((NONE, Key::Named(NamedKey::ArrowUp)), vec![Action::CursorUp]),
            ((NONE, Key::Named(NamedKey::ArrowDown)), vec![Action::CursorDown]),
            ((NONE, Key::Named(NamedKey::Delete)), vec![Action::DeleteAtCursor]),
            ((NONE, Key::Named(NamedKey::Backspace)), vec![Action::CursorLeft, Action::DeleteAtCursor]),
            ((NONE, Key::Character("=".into())), vec![Action::ReduceFontSize]),
            ((NONE, Key::Character("%".into())), vec![Action::IncreaseFontSize]),
            ((CTRL, Key::Character("z".into())), vec![Action::Undo]),
            ((CTRL, Key::Character("y".into())), vec![Action::Redo]),

            // Transition
            ((CTRL, Key::Character("1".into())), vec![Action::Transition(InputMode::Box(BoxMode))]),
            ((CTRL, Key::Character("2".into())), vec![Action::Transition(InputMode::Text(TextMode))]),
            ((CTRL, Key::Character("3".into())), vec![Action::Transition(InputMode::Color(ColorMode))]),
            ((CTRL, Key::Character("4".into())), vec![Action::Transition(InputMode::Extra(ExtraMode { buffer: Vec::new()}))]),
        ];

        m.into_iter().collect()
    };

    static ref EXTRA: InputMap = {
        #[rustfmt::skip]
        let m: Vec<((ModifiersState, Key), Vec<Action>)> = vec![
            // Controls
            ((NONE, Key::Named(NamedKey::ArrowLeft)), vec![Action::CursorLeft]),
            ((NONE, Key::Named(NamedKey::ArrowRight)), vec![Action::CursorRight]),
            ((NONE, Key::Named(NamedKey::ArrowUp)), vec![Action::CursorUp]),
            ((NONE, Key::Named(NamedKey::ArrowDown)), vec![Action::CursorDown]),
            ((NONE, Key::Named(NamedKey::Delete)), vec![Action::DeleteAtCursor]),
            ((NONE, Key::Character("=".into())), vec![Action::ReduceFontSize]),
            ((NONE, Key::Character("%".into())), vec![Action::IncreaseFontSize]),
            ((CTRL, Key::Character("z".into())), vec![Action::Undo]),
            ((CTRL, Key::Character("y".into())), vec![Action::Redo]),

            // Transition
            ((CTRL, Key::Character("1".into())), vec![Action::Transition(InputMode::Box(BoxMode))]),
            ((CTRL, Key::Character("2".into())), vec![Action::Transition(InputMode::Text(TextMode))]),
            ((CTRL, Key::Character("3".into())), vec![Action::Transition(InputMode::Color(ColorMode))]),
            ((CTRL, Key::Character("4".into())), vec![Action::Transition(InputMode::Extra(ExtraMode { buffer: Vec::new()}))]),
        ];

        m.into_iter().collect()
    };
}

pub struct Bépo;

impl KeyMap for Bépo {
    fn translate(
        &self,
        mode: InputModeIdentifier,
        modifiers: ModifiersState,
        key: Key,
    ) -> Option<&'static [Action]> {
        let map: &InputMap = match mode {
            InputModeIdentifier::Box => &BOX,
            InputModeIdentifier::Text => &TEXT,
            InputModeIdentifier::Color => &COLOR,
            InputModeIdentifier::Extra => &EXTRA,
        };

        map.get(&(modifiers, key)).map(|v| v.as_slice())
    }

    fn char_to_extra_mode_segment(&self, c: char) -> Option<Segment> {
        match c {
            'b' => Some(crate::pragmata_pro_input::SEGMENT_UP_LEFT),
            'é' => Some(crate::pragmata_pro_input::SEGMENT_UP),
            'p' => Some(crate::pragmata_pro_input::SEGMENT_UP_RIGHT),
            'a' => Some(crate::pragmata_pro_input::SEGMENT_LEFT),
            'i' => Some(crate::pragmata_pro_input::SEGMENT_RIGHT),
            'à' => Some(crate::pragmata_pro_input::SEGMENT_DOWN_LEFT),
            'y' => Some(crate::pragmata_pro_input::SEGMENT_DOWN),
            'x' => Some(crate::pragmata_pro_input::SEGMENT_DOWN_RIGHT),
            _ => None,
        }
    }

    fn extra_mode_segment_dot(&self, c: char) -> bool {
        c == 'u'
    }
}
