// Copyright The Voyager Developers 2014

#![feature(plugin)]

extern crate genmesh;
extern crate gfx;
#[macro_use]
#[plugin]
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
use noise::Brownian3;
use std::f32;
use std::mem;
use std::rand::Rng;
// use time::precise_time_s;

use camera::Camera;
use terrain::Terrain;

mod camera;
mod gen;
mod math;
mod objects;
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
- distant mountain ranges
- water
    - intersecting plane (same color as sky)
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
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::OpenglForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenglProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw
        .create_window(640, 480, "Village.", glfw::WindowMode::Windowed)
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

    let mut rng = std::rand::weak_rng();

    // Axis batch setup

    let axis_mesh   = graphics.device.create_mesh(objects::axis::VERTEX_DATA);
    let axis_slice  = axis_mesh.to_slice(gfx::PrimitiveType::Line);
    let axis_state  = gfx::DrawState::new();
    let axis_batch: shader::Batch = graphics.make_batch(&color_program, &axis_mesh, axis_slice, &axis_state).unwrap();

    // Water batch setup

    const WATER_LEVEL: f32 = -12.0;

    let water_mesh   = graphics.device.create_mesh(objects::water::VERTEX_DATA);
    let water_slice  = graphics.device.create_buffer_static(objects::water::INDEX_DATA).to_slice(gfx::PrimitiveType::TriangleList);
    let water_state = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);
    let water_batch: shader::Batch = graphics.make_batch(&color_program, &water_mesh, water_slice, &water_state).unwrap();

    // House batch setup

    let house_mesh  = graphics.device.create_mesh(objects::house::VERTEX_DATA);
    let house_slice = graphics.device.create_buffer_static(objects::house::INDEX_DATA).to_slice(gfx::PrimitiveType::TriangleList);
    let house_state = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);
    let house_batch: shader::Batch = graphics.make_batch(&flat_program, &house_mesh, house_slice, &house_state).unwrap();

    // Village generation setup

    let village_gen = gen::Scatter::new()
        .scale_non_proportional(gen::Range { min: 1.0, max: 10.0 },
                                gen::Range { min: 1.0, max: 10.0 },
                                gen::Range { min: 1.0, max: 10.0 })
        .pos_x(gen::Range { min: -100.0, max: 100.0 })
        .pos_y(gen::Range { min: -100.0, max: 100.0 });

    // Antenna batch setup

    let antenna_mesh   = graphics.device.create_mesh(objects::antenna::VERTEX_DATA);
    let antenna_slice  = antenna_mesh.to_slice(gfx::PrimitiveType::Line);
    let antenna_state  = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);
    let antenna_batch: shader::Batch = graphics.make_batch(&color_program, &antenna_mesh, antenna_slice, &antenna_state).unwrap();

    // Antenna generation setup

    let antenna_gen = gen::Scatter::new()
        .scale_non_proportional(gen::Range { min: 1.0, max: 1.0 },
                                gen::Range { min: 1.0, max: 1.0 },
                                gen::Range { min: 5.0, max: 10.0 })
        .pos_x(gen::Range { min: -100.0, max: 100.0 })
        .pos_y(gen::Range { min: -100.0, max: 100.0 });

    // Tree batch setup

    let foliage_mesh   = graphics.device.create_mesh(objects::tree::foliage::VERTEX_DATA);
    let foliage_slice  = foliage_mesh.to_slice(gfx::PrimitiveType::TriangleList);
    let foliage_state  = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);
    let foliage_batch: shader::Batch = graphics.make_batch(&color_program, &foliage_mesh, foliage_slice, &foliage_state).unwrap();

    let trunk_mesh   = graphics.device.create_mesh(objects::tree::trunk::VERTEX_DATA);
    let trunk_slice  = trunk_mesh.to_slice(gfx::PrimitiveType::Line);
    let trunk_state  = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);
    let trunk_batch: shader::Batch = graphics.make_batch(&color_program, &trunk_mesh, trunk_slice, &trunk_state).unwrap();

    // Tree generation setup

    let tree_gen = gen::Scatter::new()
        .scale_proportional(gen::Range { min: 5.0, max: 10.0 })
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
        const TERRAIN_COLOR: [f32; 3] = [0.4, 0.6, 0.2];

        let seed = rng.gen();
        let noise = Brownian3::new(noise::perlin3, 4);
        let plane = Plane::subdivide(256, 256);
        let terrain = Terrain::new(seed, noise, TERRAIN_HEIGHT_FACTOR, TERRAIN_GRID_SPACING);

        let terrain_vertices: Vec<_> = terrain
            .triangulate(plane)
            .vertices()
            .map(|(p, n)| shader::flat::Vertex {
                pos: *p.as_array(),
                norm: *n.as_array(),
                color: TERRAIN_COLOR,
            })
            .collect();

        let terrain_mesh = graphics.device.create_mesh(&*terrain_vertices);
        let terrain_slice = terrain_mesh.to_slice(gfx::PrimitiveType::TriangleList);
        let terrain_state = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);
        let terrain_batch: shader::Batch = graphics.make_batch(&flat_program, &terrain_mesh, terrain_slice, &terrain_state).unwrap();

        // Scatter objects

        let village = village_gen.scatter_objects(100, WATER_LEVEL, &terrain, &mut rng);
        let antennas = antenna_gen.scatter_objects(100, WATER_LEVEL, &terrain, &mut rng);
        let trees = tree_gen.scatter_billboards(100, WATER_LEVEL, &terrain, &mut rng);

        'event: loop {
            if window.should_close() {
                break 'main;
            }
            glfw.poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    // Close window on escape
                    glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                        window.set_should_close(true);
                    },

                    // WASD movement
                    glfw::WindowEvent::Key(glfw::Key::W, _, glfw::Action::Press, _) => cam_pos_delta.y -= KEY_DELTA,
                    glfw::WindowEvent::Key(glfw::Key::S, _, glfw::Action::Press, _) => cam_pos_delta.y += KEY_DELTA,
                    glfw::WindowEvent::Key(glfw::Key::A, _, glfw::Action::Press, _) => cam_pos_delta.x += KEY_DELTA,
                    glfw::WindowEvent::Key(glfw::Key::D, _, glfw::Action::Press, _) => cam_pos_delta.x -= KEY_DELTA,
                    // Revert WASD movement on key release
                    glfw::WindowEvent::Key(glfw::Key::W, _, glfw::Action::Release, _) => cam_pos_delta.y += KEY_DELTA,
                    glfw::WindowEvent::Key(glfw::Key::S, _, glfw::Action::Release, _) => cam_pos_delta.y -= KEY_DELTA,
                    glfw::WindowEvent::Key(glfw::Key::A, _, glfw::Action::Release, _) => cam_pos_delta.x -= KEY_DELTA,
                    glfw::WindowEvent::Key(glfw::Key::D, _, glfw::Action::Release, _) => cam_pos_delta.x += KEY_DELTA,

                    // Regenerate landscape
                    glfw::WindowEvent::Key(glfw::Key::R, _, glfw::Action::Press, _) => break 'event,

                    // Rotate camera when the cursor is moved
                    glfw::WindowEvent::CursorPos(x, y) => {
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
            let water_world = World {
                sun_dir: sun_dir,
                model: math::model_mat(Vec3::new(500.0, 500.0, 1.0),
                                       Pnt3::new(0.0, 0.0, WATER_LEVEL)),
                view_proj: view_proj,
            };

            graphics.clear(clear_data, gfx::COLOR | gfx::DEPTH, &frame);

            village.map_worlds(sun_dir, view_proj, |world| {
                graphics.draw(&house_batch, world.as_params(), &frame);
            });

            antennas.map_worlds(sun_dir, view_proj, |world| {
                graphics.draw(&antenna_batch, world.as_params(), &frame);
            });

            trees.map_worlds(sun_dir, cam, |world| {
                graphics.draw(&foliage_batch, world.as_params(), &frame);
                graphics.draw(&trunk_batch, world.as_params(), &frame);
            });

            graphics.draw(&terrain_batch, world.as_params(), &frame);
            graphics.draw(&water_batch, water_world.as_params(), &frame);
            graphics.draw(&axis_batch, world.as_params(), &frame);
            graphics.end_frame();

            window.swap_buffers();
        }
    }
}
