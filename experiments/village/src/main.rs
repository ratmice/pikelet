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

#[vertex_format]
struct Vertex {
    #[name = "a_Pos"]
    pos: [f32, ..3],

    #[name = "a_Color"]
    color: [f32, ..3],
}

impl Clone for Vertex {
    fn clone(&self) -> Vertex {
        Vertex { pos: self.pos, color: self.color }
    }
}

impl fmt::Show for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pos({}, {}, {})", self.pos[0], self.pos[1], self.pos[2])
    }
}

static VERTEX_SRC: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120

    attribute vec3 a_Pos;
    attribute vec3 a_Color;
    varying vec3 v_Color;

    uniform mat4 u_Model;
    uniform mat4 u_View;
    uniform mat4 u_Proj;

    void main() {
        v_Color = a_Color;
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
"
GLSL_150: b"
    #version 150 core

    in vec3 a_Pos;
    in vec3 a_Color;
    out vec3 v_Color;

    uniform mat4 u_Model;
    uniform mat4 u_View;
    uniform mat4 u_Proj;

    void main() {
        v_Color = a_Color;
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120

    varying vec3 v_Color;
    out vec4 o_Color;

    void main() {
        o_Color = vec4(v_Color, 1.0);
    }
"
GLSL_150: b"
    #version 150 core

    in vec3 v_Color;
    out vec4 o_Color;

    void main() {
        o_Color = vec4(v_Color, 1.0);
    }
"
};

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

    let program = graphics.device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone()).unwrap();
    let frame = gfx::Frame::new(w as u16, h as u16);
    let world = World {
        model: One::one(),
        view: {
            let mut rot: Rot3<f32> = One::one();
            rot.look_at(&Vec3 { x: 0.0, y:  0.0, z: 0.0 }, &Vec3::z());
            let mut view = to_homogeneous(&rot);
            view.set_col(3, Vec4 { x: 1.5, y: -5.0, z: 3.0, w: 0.0 });
            view
        },
        proj: PerspMat3::new(w as f32 / h as f32, 60.0 * (f32::consts::PI / 180.0), 1.0, 10.0,),
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
