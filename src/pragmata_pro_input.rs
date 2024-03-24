use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::keymap::KeyMap;

pub type Segment = u8;

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

fn chars_to_segment(c: &[char], map: &dyn KeyMap) -> Option<Segment> {
    c.iter()
        .map(|c| map.char_to_extra_mode_segment(*c))
        .reduce(|a, b| a.and_then(|a| b.map(|b| a | b)))
        .unwrap()
}

pub fn code_to_codepoint(code: &str, map: &dyn KeyMap) -> Option<u32> {
    let chars = code.chars().collect::<Vec<_>>();

    match chars.as_slice() {
        [] | [_] => None,
        ['1', a] => chars_to_segment(&[*a], map).and_then(|s| SEGMENTS.get(&s).map(|[c, _]| *c)),
        ['1', a, end] if map.extra_mode_segment_dot(*end) => {
            chars_to_segment(&[*a], map).and_then(|s| SEGMENTS.get(&s).map(|[_, c]| *c))
        }
        ['5', a, b, c, d, e] => chars_to_segment(&[*a, *b, *c, *d, *e], map)
            .and_then(|s| SEGMENTS.get(&s).map(|[c, _]| *c)),
        ['5', a, b, c, d, e, end] if map.extra_mode_segment_dot(*end) => {
            chars_to_segment(&[*a, *b, *c, *d, *e], map)
                .and_then(|s| SEGMENTS.get(&s).map(|[_, c]| *c))
        }
        _ => None,
    }
}
