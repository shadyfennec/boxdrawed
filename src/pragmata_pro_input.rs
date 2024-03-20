use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

use lazy_static::lazy_static;
use minifb::InputCallback;

pub struct DummyCallback;

impl InputCallback for DummyCallback {
    fn add_char(&mut self, _uni_char: u32) {}
}

pub enum InputAction {
    Char(char),
    Backspace,
}

pub struct ExtraInputBuffer {
    tx: Sender<InputAction>,
}

impl ExtraInputBuffer {
    pub fn new() -> (Self, Receiver<InputAction>) {
        let (tx, rx) = channel();

        (Self { tx }, rx)
    }
}

impl InputCallback for ExtraInputBuffer {
    fn add_char(&mut self, uni_char: u32) {
        if let Some(c) = char::from_u32(uni_char) {
            if c.is_ascii_alphanumeric() || c.is_ascii_whitespace() {
                self.tx
                    .send(InputAction::Char(c.to_ascii_lowercase()))
                    .unwrap();
            } else if c == 8 as char {
                self.tx.send(InputAction::Backspace).unwrap()
            }
        }
    }
}

type Segment = u8;

pub const SEGMENT_UP: Segment = 0b0000_0001;
pub const SEGMENT_UP_RIGHT: Segment = 0b0000_0010;
pub const SEGMENT_RIGHT: Segment = 0b0000_0100;
pub const SEGMENT_DOWN_RIGHT: Segment = 0b0000_1000;
pub const SEGMENT_DOWN: Segment = 0b0001_0000;
pub const SEGMENT_DOWN_LEFT: Segment = 0b0010_0000;
pub const SEGMENT_LEFT: Segment = 0b0100_0000;
pub const SEGMENT_UP_LEFT: Segment = 0b1000_0000;

lazy_static! {
    static ref SEGMENTS: HashMap<Segment, [u32; 2]> = {
        #[rustfmt::skip]
        let map: &[(Segment, [u32; 2])] = &[
            // 1 segment
            (SEGMENT_UP, [0x2575, 0x1004EE]),

            // 5 segments
            (SEGMENT_UP | SEGMENT_UP_LEFT | SEGMENT_UP_RIGHT | SEGMENT_DOWN_LEFT | SEGMENT_DOWN_RIGHT, [0x100420, 0x1005C0]),
        ];

        map.iter().copied().collect()
    };
}

fn char_to_segment(c: char) -> Option<Segment> {
    match c {
        'a' => Some(SEGMENT_UP_LEFT),
        'z' => Some(SEGMENT_UP),
        'e' => Some(SEGMENT_UP_RIGHT),
        'q' => Some(SEGMENT_LEFT),
        'd' => Some(SEGMENT_RIGHT),
        'w' => Some(SEGMENT_DOWN_LEFT),
        'x' => Some(SEGMENT_DOWN),
        'c' => Some(SEGMENT_DOWN_RIGHT),
        _ => None,
    }
}

fn chars_to_segment(c: &[char]) -> Option<Segment> {
    c.iter()
        .map(|c| char_to_segment(*c))
        .reduce(|a, b| a.and_then(|a| b.map(|b| a | b)))
        .unwrap()
}

pub fn code_to_codepoint(code: &str) -> Option<u32> {
    let chars = code.chars().collect::<Vec<_>>();

    match chars.as_slice() {
        [] | [_] => None,
        ['1', a] => chars_to_segment(&[*a]).and_then(|s| SEGMENTS.get(&s).map(|[c, _]| *c)),
        ['1', a, 's'] => chars_to_segment(&[*a]).and_then(|s| SEGMENTS.get(&s).map(|[_, c]| *c)),
        ['5', a, b, c, d, e] => {
            chars_to_segment(&[*a, *b, *c, *d, *e]).and_then(|s| SEGMENTS.get(&s).map(|[c, _]| *c))
        }
        ['5', a, b, c, d, e, 's'] => {
            chars_to_segment(&[*a, *b, *c, *d, *e]).and_then(|s| SEGMENTS.get(&s).map(|[_, c]| *c))
        }
        _ => None,
    }
}
