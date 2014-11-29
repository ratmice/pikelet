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
use std::mem;
use std::rand::Rng;
// use time::precise_time_s;

use camera::Camera;
use terrain::Terrain;

mod axis_thingy;
mod camera;
mod forest;
mod gen;
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

struct World {
    pub sun_dir: Vec3<f32>,
    pub model: Mat4<f32>,
    pub view_proj: Mat4<f32>,
}

impl World {
    pub fn as_params(&self) -> &shader::Params {
        unsafe { mem::transmute(self) }
    }
}

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
    window.set_key_polling(true);

    window.set_cursor_pos_polling(true);

    // Graphics setup

    let device          = gfx::GlDevice::new(|s| window.get_proc_address(s));
    let mut graphics    = gfx::Graphics::new(device);

    let color_program   = graphics.device.link_program(shader::color::VERTEX_SRC.clone(),
                                                       shader::color::FRAGMENT_SRC.clone()).unwrap();
    let flat_program    = graphics.device.link_program(shader::flat::VERTEX_SRC.clone(),
                                                       shader::flat::FRAGMENT_SRC.clone()).unwrap();
    let frame           = gfx::Frame::new(w as u16, h as u16);
    let clear_data      = gfx::ClearData { color: sky::DAY_COLOR, depth: 1.0, stencil: 0 };

    // RNG setup

    let mut rng = std::rand::task_rng();

    // Axis batch setup

    let axis_mesh   = graphics.device.create_mesh(axis_thingy::VERTEX_DATA);
    let axis_slice  = axis_mesh.to_slice(gfx::Line);
    let axis_state  = gfx::DrawState::new();
    let axis_batch: shader::Batch = graphics.make_batch(&color_program, &axis_mesh, axis_slice, &axis_state).unwrap();

    // House batch setup

    let house_mesh  = graphics.device.create_mesh(house::VERTEX_DATA);
    let house_slice = graphics.device.create_buffer_static(house::INDEX_DATA).to_slice(gfx::TriangleList);
    let house_state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);
    let house_batch: shader::Batch = graphics.make_batch(&flat_program, &house_mesh, house_slice, &house_state).unwrap();

    // Village generation setup

    let village_gen = gen::Scatter::new()
        .scale_x(gen::Range { min: 1.0, max: 10.0 })
        .scale_y(gen::Range { min: 1.0, max: 10.0 })
        .scale_z(gen::Range { min: 1.0, max: 10.0 })
        .pos_x(gen::Range { min: -100.0, max: 100.0 })
        .pos_y(gen::Range { min: -100.0, max: 100.0 });

    'main: loop {
        // Camera stuff

        let aspect = w as f32 / h as f32;
        let fov = 45.0 * (f32::consts::PI / 180.0);
        let proj = PerspMat3::new(aspect, fov, 0.1, 300.0);
        let mut cam = Camera::new(zero(), proj);
        cam.look_at(&Pnt3::new(5.0, 20.0, 10.0), &Pnt3::new(0.0,  0.0, 0.0), &Vec3::z());

        const KEY_DELTA: f32 = 0.5;
        let mut cam_pos_delta: Vec3<f32> = zero();

        // Initialise the first cursor position. This will help us calculate the
        // delta later.
        let mut cursor_prev = {
            let (x, y) = window.get_cursor_pos();
            Pnt2::new(x as f32, y as f32)
        };

        // Terrain

        const TERRAIN_HEIGHT_FACTOR: f32 = 100.0;
        const TERRAIN_GRID_SPACING: f32 = 1200.0;
        const TERRAIN_COLOR: [f32, ..3] = [0.4, 0.6, 0.2];

        let rand_seed = rng.gen();
        let noise = Perlin::new().seed(rand_seed).frequency(10.0);
        let plane = Plane::subdivide(256, 256);
        let terrain = Terrain::new(TERRAIN_HEIGHT_FACTOR, TERRAIN_GRID_SPACING, noise);

        let terrain_vertices: Vec<_> = terrain
            .triangulate(plane)
            .vertices()
            .map(|(p, n)| shader::flat::Vertex {
                pos: *p.as_array(),
                norm: *n.as_array(),
                color: TERRAIN_COLOR,
            })
            .collect();

        let terrain_mesh = graphics.device.create_mesh(terrain_vertices.as_slice());
        let terrain_slice = terrain_mesh.to_slice(gfx::TriangleList);
        let terrain_state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);
        let terrain_batch: shader::Batch = graphics.make_batch(&flat_program, &terrain_mesh, terrain_slice, &terrain_state).unwrap();

        // Scatter houses

        let village = village_gen.scatter(100, &terrain, &mut rng);

        'event: loop {
            if window.should_close() {
                break 'main;
            }
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

                    // Regenerate landscape
                    glfw::KeyEvent(glfw::Key::R, _, glfw::Press, _) => break 'event,

                    // Rotate camera when the cursor is moved
                    glfw::CursorPosEvent(x, y) => {
                        let cursor_curr = Pnt2::new(x as f32, y as f32);
                        let cursor_delta = cursor_prev - cursor_curr;
                        cursor_prev = cursor_curr;
                        let _ = cursor_delta; // unused
                        // println!("{}", cursor_delta);
                    },

                    // Everything else
                    _ => {},
                }
            }

            cam.view.append_translation(&cam_pos_delta);

            let sun_dir = Vec3::new(0.0, 0.5, 1.0);
            let view_proj = cam.to_mat();
            let world = World {
                sun_dir: sun_dir,
                model: one(),
                view_proj: view_proj,
            };

            graphics.clear(clear_data, gfx::COLOR | gfx::DEPTH, &frame);

            village.map_worlds(sun_dir, view_proj, |world| {
                graphics.draw(&house_batch, world.as_params(), &frame);
            });

            graphics.draw(&terrain_batch, world.as_params(), &frame);
            graphics.draw(&axis_batch, world.as_params(), &frame);
            graphics.end_frame();

            window.swap_buffers();
        }
    }
}
