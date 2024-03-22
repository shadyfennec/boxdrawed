use std::{
    collections::HashMap,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Coordinates {
    pub x: isize,
    pub y: isize,
}

impl Coordinates {
    pub fn min(&self, value: isize) -> Self {
        (self.x.min(value), self.y.min(value)).into()
    }

    pub fn max(&self, value: isize) -> Self {
        (self.x.max(value), self.y.max(value)).into()
    }

    pub fn unsigned_abs(&self) -> (usize, usize) {
        (self.x.unsigned_abs(), self.y.unsigned_abs())
    }
}

impl Mul for Coordinates {
    type Output = Coordinates;

    fn mul(self, rhs: Self) -> Self::Output {
        (self.x * rhs.x, self.y * rhs.y).into()
    }
}

impl Mul<isize> for Coordinates {
    type Output = Coordinates;

    fn mul(self, rhs: isize) -> Self::Output {
        (self.x * rhs, self.y * rhs).into()
    }
}

impl MulAssign<isize> for Coordinates {
    fn mul_assign(&mut self, rhs: isize) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Add for Coordinates {
    type Output = Coordinates;

    fn add(self, rhs: Self) -> Self::Output {
        (self.x + rhs.x, self.y + rhs.y).into()
    }
}

impl AddAssign<Coordinates> for Coordinates {
    fn add_assign(&mut self, rhs: Coordinates) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Coordinates {
    type Output = Coordinates;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y).into()
    }
}

impl SubAssign<Coordinates> for Coordinates {
    fn sub_assign(&mut self, rhs: Coordinates) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub top_left: Coordinates,
    pub width: usize,
    pub height: usize,
}

impl BoundingBox {
    pub fn new<C>(top_left: C, width: usize, height: usize) -> Self
    where
        C: Into<Coordinates>,
    {
        Self {
            top_left: top_left.into(),
            width,
            height,
        }
    }

    pub fn contains(&self, point: &Coordinates) -> bool {
        (self.top_left.x..self.top_left.x.saturating_add_unsigned(self.width)).contains(&point.x)
            && (self.top_left.y..self.top_left.y.saturating_add_unsigned(self.height))
                .contains(&point.y)
    }

    pub fn convert_to_relative(&self, point: &Coordinates) -> Option<Coordinates> {
        if self.contains(point) {
            assert!(point.x - self.top_left.x >= 0);
            assert!(point.y - self.top_left.y >= 0);
            Some((point.x - self.top_left.x, point.y - self.top_left.y).into())
        } else {
            None
        }
    }
}

impl<C> From<(C, usize, usize)> for BoundingBox
where
    C: Into<Coordinates>,
{
    fn from((top_left, width, height): (C, usize, usize)) -> Self {
        Self::new(top_left, width, height)
    }
}

impl From<(isize, isize)> for Coordinates {
    fn from((x, y): (isize, isize)) -> Self {
        Coordinates { x, y }
    }
}

struct TextStorage {
    characters: HashMap<Coordinates, char>,
}

impl TextStorage {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
        }
    }

    pub fn characters_in_bounding_box<B>(
        &self,
        bounding_box: B,
    ) -> impl Iterator<Item = (Coordinates, char)> + '_
    where
        B: Into<BoundingBox>,
    {
        let bounding_box = bounding_box.into();

        self.characters.iter().filter_map(move |(coords, c)| {
            bounding_box
                .convert_to_relative(coords)
                .map(|coords| (coords, *c))
        })
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn vector(&self) -> Coordinates {
        match self {
            Direction::Up => (0, -1),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
        }
        .into()
    }
}

pub struct TextArea {
    text_storage: TextStorage,
    bounding_box: BoundingBox,
    cursor_absolute_position: Coordinates,
    view_cache: Option<Vec<(Coordinates, char)>>,
}

impl TextArea {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            text_storage: TextStorage::new(),
            bounding_box: (Coordinates::from((0, 0)), width, height).into(),
            cursor_absolute_position: (0, 0).into(),
            view_cache: None,
        }
    }

    pub fn cursor_relative_position(&self) -> Coordinates {
        (
            self.cursor_absolute_position.x - self.bounding_box.top_left.x,
            self.cursor_absolute_position.y - self.bounding_box.top_left.y,
        )
            .into()
    }

    pub fn ensure_cache(&mut self) {
        if self.view_cache.is_none() {
            self.view_cache = Some(
                self.text_storage
                    .characters_in_bounding_box(self.bounding_box)
                    .collect(),
            )
        }
    }

    pub fn chars(&mut self) -> impl Iterator<Item = (Coordinates, char)> + '_ {
        self.ensure_cache();

        self.view_cache.as_ref().unwrap().iter().copied()
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        self.cursor_absolute_position += direction.vector();
        self.view_cache = None;
    }

    pub fn move_cursor_by(&mut self, direction: Direction, amount: usize) {
        self.cursor_absolute_position += direction.vector() * amount as isize;
        self.view_cache = None;
    }

    pub fn write_at_cursor(&mut self, c: char) {
        self.text_storage
            .characters
            .entry(self.cursor_absolute_position)
            .and_modify(|x| *x = c)
            .or_insert(c);

        self.view_cache = None;
    }

    pub fn erase_at_cursor(&mut self) {
        self.text_storage
            .characters
            .remove(&self.cursor_absolute_position);

        self.view_cache = None;
    }

    pub fn string_at(&self, mut start: Coordinates) -> String {
        let mut s = String::new();

        while let Some(c) = self.text_storage.characters.get(&start) {
            s.push(*c);
            start += (1, 0).into();
        }

        s
    }

    pub fn clear(&mut self) {
        self.text_storage.characters.clear();
        self.view_cache = None;
    }

    pub fn write_string_at_cursor(&mut self, s: &str) {
        for c in s.chars() {
            self.write_at_cursor(c);
            self.move_cursor(Direction::Right);
        }
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_absolute_position = (0, 0).into();
    }

    pub fn replace_with_string(&mut self, s: &str) {
        self.clear();
        self.reset_cursor();
        self.write_string_at_cursor(s);
    }
}