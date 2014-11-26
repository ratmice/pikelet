// Copyright Brendan Zabarauskas 2014

#![feature(default_type_params)]
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
use genmesh::{Triangulate, Vertices};
use genmesh::generators::Plane;
use nalgebra::*;
use noise::source::Perlin;
use std::f32;
use std::rand::Rng;
// use time::precise_time_s;

use camera::Camera;
use terrain::Terrain;

mod axis_thingy;
mod camera;
mod forest;
mod house;
mod shader;
mod sky;
mod terrain;

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

    // Graphics stuff

    let device          = gfx::GlDevice::new(|s| window.get_proc_address(s));
    let mut graphics    = gfx::Graphics::new(device);
    let program         = graphics.device.link_program(shader::VERTEX_SRC.clone(),
                                                       shader::FRAGMENT_SRC.clone()).unwrap();
    let frame           = gfx::Frame::new(w as u16, h as u16);
    let clear_data      = gfx::ClearData { color: sky::DAY_COLOR, depth: 1.0, stencil: 0 };

    // Camera stuff

    let aspect = w as f32 / h as f32;
    let fov = 45.0 * (f32::consts::PI / 180.0);
    let proj = PerspMat3::new(aspect, fov, 1.0, 10.0);
    let mut cam = Camera::new(zero(), proj);
    cam.look_at(&Pnt3::new(1.5, 5.0, 3.0), &Pnt3::new(0.0,  0.0, 0.0), &Vec3::z());

    const KEY_DELTA: f32 = 0.1;
    let mut cam_pos_delta: Vec3<f32> = zero();

    // House

    let house_mesh  = graphics.device.create_mesh(house::VERTEX_DATA);
    let house_slice = graphics.device.create_buffer_static(house::INDEX_DATA).to_slice(gfx::TriangleList);
    let house_state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);
    let house_batch: shader::Batch = graphics.make_batch(&program, &house_mesh, house_slice, &house_state).unwrap();

    // Terrain

    const TERRAIN_HEIGHT_FACTOR: f32 = 5.0;
    const TERRAIN_GRID_SPACING: f32 = 30.0;
    const TERRAIN_COLOR: [f32, ..3] = [0.4, 0.6, 0.2];

    let rand_seed = std::rand::task_rng().gen();
    let noise = Perlin::new().seed(rand_seed);
    let plane = Plane::subdivide(256, 256);
    let terrain = Terrain::new(TERRAIN_HEIGHT_FACTOR, TERRAIN_GRID_SPACING, noise);

    let terrain_vertices: Vec<_> = terrain
        .triangulate(plane)
        .vertices()
        .map(|(p, _)| shader::Vertex {
            pos: *p.as_array(),
            // normal: *n.as_array(),
            color: TERRAIN_COLOR,
        })
        .collect();

    let terrain_mesh = graphics.device.create_mesh(terrain_vertices.as_slice());
    let terrain_slice = terrain_mesh.to_slice(gfx::TriangleList);
    let terrain_state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);
    let terrain_batch: shader::Batch = graphics.make_batch(&program, &terrain_mesh, terrain_slice, &terrain_state).unwrap();

    // Axis

    let axis_mesh   = graphics.device.create_mesh(axis_thingy::VERTEX_DATA);
    let axis_slice  = axis_mesh.to_slice(gfx::Line);
    let axis_state  = gfx::DrawState::new();
    let axis_batch: shader::Batch = graphics.make_batch(&program, &axis_mesh, axis_slice, &axis_state).unwrap();

    // Main loop

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                // Close window on escape
                glfw::KeyEvent(glfw::Key::Escape, _, glfw::Press, _) => {
                    window.set_should_close(true);
                },

                // WASD movement
                glfw::KeyEvent(glfw::Key::W, _, glfw::Press, _) => cam_pos_delta.y -= KEY_DELTA,
                glfw::KeyEvent(glfw::Key::S, _, glfw::Press, _) => cam_pos_delta.y += KEY_DELTA,
                glfw::KeyEvent(glfw::Key::A, _, glfw::Press, _) => cam_pos_delta.x += KEY_DELTA,
                glfw::KeyEvent(glfw::Key::D, _, glfw::Press, _) => cam_pos_delta.x -= KEY_DELTA,
                // Revert WASD movement on key release
                glfw::KeyEvent(glfw::Key::W, _, glfw::Release, _) => cam_pos_delta.y += KEY_DELTA,
                glfw::KeyEvent(glfw::Key::S, _, glfw::Release, _) => cam_pos_delta.y -= KEY_DELTA,
                glfw::KeyEvent(glfw::Key::A, _, glfw::Release, _) => cam_pos_delta.x -= KEY_DELTA,
                glfw::KeyEvent(glfw::Key::D, _, glfw::Release, _) => cam_pos_delta.x += KEY_DELTA,

                // Everything else
                _ => {},
            }
        }

        cam.view.append_translation(&cam_pos_delta);
        let world = cam.to_mat();
        let params = shader::Params { transform: *world.as_array() };

        graphics.clear(clear_data, gfx::COLOR | gfx::DEPTH, &frame);
        graphics.draw(&house_batch, &params, &frame);
        graphics.draw(&terrain_batch, &params, &frame);
        graphics.draw(&axis_batch, &params, &frame);
        graphics.end_frame();

        window.swap_buffers();
    }
}
