// Copyright Brendan Zabarauskas 2014

#![feature(globs)]
#![feature(phase)]

extern crate genmesh;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
// extern crate glutin;
extern crate glfw;
extern crate nalgebra;
extern crate noise;
extern crate time;

use gfx::{Device, DeviceHelper, ToSlice};
use glfw::Context;
// use genmesh::{Vertices, Triangulate};
// use genmesh::generators::{Plane, SharedVertex, IndexedPolygon};
use nalgebra::*;
use std::f32;
use std::fmt;
use std::rand::Rng;
// use time::precise_time_s;

// use noise::source::Perlin;
// use noise::source::Source;

use world::{World, WorldBatch};

mod axis_thingy;
mod camera;
mod forest;
mod house;
mod shader;
mod sky;
mod world;

////////////////////////////////////////////////////////////////////////////////
/*******************************************************************************

TODO:

- first person, freelook camera
- scattered objects
    - houses
        - cubes
        - gabled
        - on stilts
    - trees
    - antennae
    - flags
    - standing stones
    - fences
    - hedges
    - stone walls
    - roads/paths
- perlin terrain
- distant mountain ranges
- water
    - intersecting plane (same color as sky)
- shadow shader based on sun direction
- sky
    - sun
    - day/night
    - stars
    - moon
    - clouds

*******************************************************************************/
////////////////////////////////////////////////////////////////////////////////

fn main() {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::ContextVersion(3, 2));
    glfw.window_hint(glfw::OpenglForwardCompat(true));
    glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlProfileHint::Core));

    let (window, events) = glfw
        .create_window(640, 480, "Village.", glfw::Windowed)
        .expect("Failed to create GLFW window.");

    let (w, h) = window.get_framebuffer_size();

    window.make_current();
    glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
    window.set_key_polling(true);

    let device          = gfx::GlDevice::new(|s| window.get_proc_address(s));
    let mut graphics    = gfx::Graphics::new(device);

    let program = graphics.device.link_program(shader::VERTEX_SRC.clone(),
                                               shader::FRAGMENT_SRC.clone()).unwrap();
    let frame = gfx::Frame::new(w as u16, h as u16);
    let world = World {
        model: One::one(),
        view: to_homogeneous(&{
            let mut transform = one::<Iso3<f32>>();
            transform.look_at_z(&Pnt3::new(1.5, -5.0, 3.0),
                                &Pnt3::new(0.0,  0.0, 0.0),
                                &Vec3::z());
            inv(&transform).unwrap()
        }),
        proj: PerspMat3::new(w as f32 / h as f32,
                             45.0 * (f32::consts::PI / 180.0),
                             1.0, 10.0),
    };

    // let house_mesh      = graphics.device.create_mesh(house::VERTEX_DATA);
    // let house_slice     = graphics.device.create_buffer_static(house::INDEX_DATA).to_slice(gfx::TriangleList);
    // let house_state     = gfx::DrawState::new().depth(gfx::state::LessEqual, true);
    // let house_batch: WorldBatch = graphics.make_batch(&program, &house_mesh, house_slice, &house_state).unwrap();

    let axis_mesh       = graphics.device.create_mesh(axis_thingy::VERTEX_DATA);
    let axis_slice      = axis_mesh.to_slice(gfx::Line);
    let axis_state      = gfx::DrawState::new().depth(gfx::state::LessEqual, true);
    let axis_batch: WorldBatch = graphics.make_batch(&program, &axis_mesh, axis_slice, &axis_state).unwrap();

    let clear_data      = gfx::ClearData { color: sky::DAY_COLOR, depth: 1.0, stencil: 0 };

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::KeyEvent(glfw::Key::Escape, _, glfw::Press, _) =>
                    window.set_should_close(true),
                _ => {},
            }
        }

        graphics.clear(clear_data, gfx::COLOR | gfx::DEPTH, &frame);
        graphics.draw(&axis_batch, world.as_params(), &frame);
        // graphics.draw(&house_batch, world.as_params(), &frame);
        graphics.end_frame();

        window.swap_buffers();
    }
}
