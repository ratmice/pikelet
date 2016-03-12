use glium::backend::Facade;
use glium::texture::{ClientFormat, RawImage2d, Texture2d};
use rusttype::{self, Font, Pixels};
use std::borrow::Cow;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub fn draw<F>(facade: &F, font: &Font, text: &str, height: f32) -> Texture2d where
    F: Facade,
{
    let pixel_height = height.ceil() as usize;
    let scale = Pixels(height);

    let v_metrics = font.v_metrics(scale);
    let offset = rusttype::point(0.0, v_metrics.ascent);

    let glyphs: Vec<_> = font.layout(&text, scale, offset).collect();

    let width = glyphs.iter()
        .map(|glyph| glyph.h_metrics().advance_width)
        .fold(0.0, |x, y| x + y)
        .ceil() as usize;

    println!("width: {}, height: {}", width, pixel_height);

    let mut data = vec![0.0; width * pixel_height];
    for (i, glyph) in glyphs.iter().enumerate() {
        glyph.draw(|_, _, value| data[i] = value as f32);
    }

    println!("data.len(): {}, width x height: {}", data.len(), width * pixel_height);

    let raw_image = RawImage2d {
        data: Cow::Borrowed(&data),
        width: width as u32,
        height: pixel_height as u32,
        format: ClientFormat::F32,
    };

    Texture2d::new(facade, raw_image).unwrap()
}