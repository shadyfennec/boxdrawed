use std::{collections::HashMap, num::NonZeroU32, rc::Rc};

use fontdue::{layout::GlyphRasterConfig, Font, Metrics};
use softbuffer::Surface;
use winit::window::Window;

use crate::text_area::{BoundingBox, Coordinates, TextArea};

type FontAtlas = HashMap<GlyphRasterConfig, (Metrics, Vec<u8>)>;

struct FontManager {
    font: Font,
    atlas: FontAtlas,
    font_size: f32,
}

impl FontManager {
    pub fn new(font: Font, font_size: f32) -> Self {
        Self {
            font,
            atlas: HashMap::new(),
            font_size,
        }
    }

    pub fn character_height(&self) -> usize {
        self.font_size as usize
    }

    pub fn character_width(&self) -> usize {
        // TODO: big assumption here
        self.character_height() / 2
    }

    fn config_of(&self, c: char) -> GlyphRasterConfig {
        GlyphRasterConfig {
            glyph_index: self.font.lookup_glyph_index(c),
            px: self.font_size,
            font_hash: self.font.file_hash(),
        }
    }

    pub fn rasterize(&mut self, c: char) -> &(Metrics, Vec<u8>) {
        self.atlas
            .entry(self.config_of(c))
            .or_insert(self.font.rasterize(c, self.font_size))
    }
}

struct FrameBuffer {
    width: usize,
    height: usize,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    fn lines<'a, B>(
        &'a self,
        bounding_box: B,
        buffer: &'a mut [u32],
    ) -> impl Iterator<Item = &'a mut [u32]> + '_
    where
        B: Into<BoundingBox>,
    {
        let bounding_box = bounding_box.into();

        buffer
            .chunks_mut(self.width)
            .skip(bounding_box.top_left.y as usize)
            .take(bounding_box.height)
            .map(move |l| {
                let start = bounding_box.top_left.x as usize;

                &mut l[start..start + bounding_box.width]
            })
    }

    pub fn draw<I, C>(&mut self, chars: I, font: &mut FontManager, top_left: C, buffer: &mut [u32])
    where
        I: IntoIterator<Item = (Coordinates, char)>,
        C: Into<Coordinates>,
    {
        let top_left = top_left.into();
        let char_width = font.character_width();
        let char_height = font.character_height();

        for (coord, c) in chars.into_iter() {
            let (metrics, bitmap) = font.rasterize(c);

            if !bitmap.is_empty() {
                let mut character_top_left = (coord + top_left)
                    * Coordinates::from((char_width as isize, char_height as isize));

                character_top_left.y +=
                    char_height as isize - metrics.ymin as isize - metrics.height as isize;
                character_top_left.x += metrics.xmin as isize;

                let bounded_top_left = character_top_left.max(0);
                let (displacement_width, displacement_height) =
                    character_top_left.min(0).unsigned_abs();

                let displayed_width = metrics
                    .width
                    .min(self.width - bounded_top_left.x.max(0).unsigned_abs());
                let displayed_height = metrics
                    .height
                    .min(self.width - bounded_top_left.y.max(0).unsigned_abs());

                for (dest_line, src_line) in self
                    .lines(
                        (
                            (bounded_top_left),
                            displayed_width - displacement_width,
                            displayed_height - displacement_height,
                        ),
                        buffer,
                    )
                    .zip(
                        bitmap
                            .chunks(metrics.width)
                            .skip(displacement_height)
                            .take(displayed_height),
                    )
                {
                    let src = src_line[displacement_width
                        ..displacement_width + (displayed_width - displacement_width)]
                        .iter()
                        .map(|v| u32::from_be_bytes([0, *v, *v, *v]))
                        .collect::<Vec<_>>();

                    for (dest, src) in dest_line.iter_mut().zip(src) {
                        let mut d = dest.to_be_bytes();
                        let s = src.to_be_bytes();

                        if d != s {
                            d[1] = d[1].saturating_add(s[1]);
                            d[2] = d[2].saturating_add(s[2]);
                            d[3] = d[3].saturating_add(s[3]);

                            *dest = u32::from_be_bytes(d);
                        }
                    }
                }
            }
        }
    }

    pub fn invert<B, C>(
        &mut self,
        bounding_box: B,
        top_left: C,
        font: &FontManager,
        buffer: &mut [u32],
    ) where
        B: Into<BoundingBox>,
        C: Into<Coordinates>,
    {
        let bounding_box = bounding_box.into();
        let top_left = top_left.into();
        let char_width = font.character_width();
        let char_height = font.character_height();

        let top_left = top_left + bounding_box.top_left;

        for coord in (top_left.y..top_left.y + bounding_box.height as isize).flat_map(|y| {
            (top_left.x..top_left.x + bounding_box.width as isize)
                .map(move |x| Coordinates::from((x, y)))
        }) {
            let character_top_left =
                coord * Coordinates::from((char_width as isize, char_height as isize));

            for dest_line in self.lines((character_top_left, char_width, char_height), buffer) {
                for v in dest_line {
                    let [x, a, b, c] = v.to_be_bytes();

                    *v = u32::from_be_bytes([x, !a, !b, !c]);
                }
            }
        }
    }
}

pub struct Canvas {
    frame_buffer: FrameBuffer,
    surface: Surface<Rc<Window>, Rc<Window>>,
    font: FontManager,
    pub top_line: TextArea,
    pub draw_area: TextArea,
    pub bottom_line: TextArea,
}

impl Canvas {
    pub fn new(
        font: Font,
        font_size: f32,
        surface: Surface<Rc<Window>, Rc<Window>>,
        width: usize,
        height: usize,
    ) -> Self {
        let font = FontManager::new(font, font_size);

        let top_line = TextArea::new(width / font.character_width(), 1);
        let draw_area = TextArea::new(
            width / font.character_width(),
            (height / font.character_height()) - 2,
        );
        let bottom_line = TextArea::new(width / font.character_width(), 1);

        Self {
            frame_buffer: FrameBuffer::new(width, height),
            surface,
            font,
            top_line,
            draw_area,
            bottom_line,
        }
    }

    pub fn width(&self) -> usize {
        self.frame_buffer.width
    }

    pub fn height(&self) -> usize {
        self.frame_buffer.height
    }

    pub fn render(&mut self) {
        let char_height = self.font.character_height();
        let mut buffer = self.surface.buffer_mut().unwrap();

        for v in buffer.iter_mut() {
            *v = 0
        }

        self.top_line.bounding_box.top_left = (0, 0).into();
        self.bottom_line.bounding_box.top_left = (0, 0).into();

        self.frame_buffer
            .draw(self.top_line.chars(), &mut self.font, (0, 0), &mut buffer);

        self.frame_buffer
            .draw(self.draw_area.chars(), &mut self.font, (0, 1), &mut buffer);

        self.frame_buffer.draw(
            self.bottom_line.chars(),
            &mut self.font,
            (
                0,
                ((self.frame_buffer.height / char_height).saturating_sub(1)) as _,
            ),
            &mut buffer,
        );

        // make cursor inverted
        self.frame_buffer.invert(
            ((self.draw_area.cursor_relative_position()), 1, 1),
            (0, 1),
            &self.font,
            &mut buffer,
        );

        buffer.present().unwrap();
    }

    pub fn font_size(&self) -> f32 {
        self.font.font_size
    }

    pub fn set_font_size(&mut self, value: f32) {
        let old_value = self.font.font_size;
        self.font.font_size = value;

        if self.font.character_height() > self.height() / 2
            || self.font.character_width() > self.width() / 2
        {
            self.font.font_size = old_value
        } else {
            self.top_line
                .set_size(self.width() / self.font.character_width(), 1);
            self.draw_area.set_size(
                self.width() / self.font.character_width(),
                (self.height() / self.font.character_height()).saturating_sub(1),
            );
            self.bottom_line
                .set_size(self.width() / self.font.character_width(), 1);
        }
    }

    pub fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) {
        self.surface.resize(width, height).unwrap();

        let width = width.get() as usize;
        let height = height.get() as usize;

        self.frame_buffer.width = width;
        self.frame_buffer.height = height;

        self.top_line
            .set_size(width / self.font.character_width(), 1);
        self.draw_area.set_size(
            width / self.font.character_width(),
            (height / self.font.character_height()).saturating_sub(1),
        );
        self.bottom_line
            .set_size(width / self.font.character_width(), 1);
    }
}
