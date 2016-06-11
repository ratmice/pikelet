use cgmath::{Matrix4, Point2};
use glium::texture::{ClientFormat, RawImage2d, Texture2dDataSource};
use rusttype::{self, Font, Scale};
use std::borrow::Cow;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub const TEXTURE_HEIGHT: f32 = 1.0;
pub const TEXTURE_WIDTH: f32 = 1.0;

pub const TEXTURE_VERTICES: [Vertex; 4] = [
    Vertex { position: [          0.0,            0.0], tex_coords: [0.0, 0.0] }, // Top-left
    Vertex { position: [TEXTURE_WIDTH,            0.0], tex_coords: [1.0, 0.0] }, // Top-right
    Vertex { position: [TEXTURE_WIDTH, TEXTURE_HEIGHT], tex_coords: [1.0, 1.0] }, // Bottom-right
    Vertex { position: [          0.0, TEXTURE_HEIGHT], tex_coords: [0.0, 1.0] }, // Bottom-left
];

pub const TEXTURE_INDICES: [u8; 2 * 3] = [
    0, 1, 2,
    2, 3, 0,
];

#[derive(Clone, Debug)]
pub struct TextData {
    data: Vec<f32>,
    pub width: u32,
    pub height: u32,
}

impl TextData {
    pub fn new(font: &Font, text: &str, height: f32) -> TextData {
        let pixel_height = height.ceil() as usize;
        let scale = Scale::uniform(height);

        let v_metrics = font.v_metrics(scale);
        let offset = rusttype::point(0.0, v_metrics.ascent);

        let glyphs: Vec<_> = font.layout(text, scale, offset).collect();

        let width = glyphs.iter()
            .map(|glyph| glyph.unpositioned().h_metrics().advance_width)
            .fold(0.0, |x, y| x + y)
            .ceil() as usize;

        let mut data = vec![0.0; width * pixel_height];
        for glyph in glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, value| {
                    let x = (x as i32 + bb.min.x) as usize;
                    let y = (y as i32 + bb.min.y) as usize;
                    data[x + y * width] = value as f32;
                });
            }
        }

        TextData {
            data: data,
            width: width as u32,
            height: pixel_height as u32,
        }
    }

    pub fn matrix(&self, position: Point2<f32>) -> Matrix4<f32> {
        let scale_x = self.width as f32 / TEXTURE_WIDTH;
        let scale_y = self.height as f32 / TEXTURE_HEIGHT;
        let mut matrix = Matrix4::from_nonuniform_scale(scale_x, scale_y, 1.0);

        matrix[3][0] = position.x;
        matrix[3][1] = position.y;

        matrix
    }
}

impl<'a> Texture2dDataSource<'a> for &'a TextData {
    type Data = f32;

    fn into_raw(self) -> RawImage2d<'a, f32> {
        RawImage2d {
            data: Cow::Borrowed(&self.data),
            width: self.width,
            height: self.height,
            format: ClientFormat::F32,
        }
    }
}
