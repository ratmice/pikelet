extern crate cgmath;
#[macro_use] extern crate glium;
extern crate time;

pub use glium::glutin;

use cgmath::{Angle, PerspectiveFov, Rad};
use cgmath::{Matrix4, SquareMatrix};
use cgmath::{Point3, Point, Vector3};
use glium::{BackfaceCullingMode, Depth, DepthTest, DrawParameters};
use glium::{DisplayBuild, PolygonMode, Program, Surface, VertexBuffer};
use glium::index::{PrimitiveType, NoIndices};
use glutin::{ElementState, Event, WindowBuilder};
use glutin::VirtualKeyCode as Key;

use camera::Camera;

pub mod camera;
pub mod color;
pub mod math;
pub mod polyhedra;
pub mod times;

const WINDOW_TITLE: &'static str = "Geodesic Test";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 500;

const CAMERA_XZ_RADIUS: f32 = 2.0;
const CAMERA_Y_HEIGHT: f32 = 1.0;
const CAMERA_NEAR: f32 = 0.1;
const CAMERA_FAR: f32 = 300.0;

const POLYHEDRON_SUBDIVS: usize = 3;
const POLYHEDRON_RADIUS: f32 = 1.0;

const LIGHT_DIR: Vector3<f32> = Vector3 { x: 0.0, y: 0.5, z: 1.0 };
const ROTATIONS_PER_SECOND: f32 = 0.1;

#[derive(Copy, Clone)]
pub struct Vertex {
    normal: [f32; 3],
    position: [f32; 3],
}

implement_vertex!(Vertex, normal, position);

#[derive(Clone, Debug)]
pub struct Geometry {
    pub points: Vec<Point3<f32>>,
    pub edges: Vec<[usize; 2]>,
    pub faces: Vec<[usize; 3]>,
}

impl Geometry {
    pub fn subdivide(&self, radius: f32, count: usize) -> Geometry {
        (0..count).fold(self.clone(), |acc, _| acc.subdivide_once(radius))
    }

    pub fn subdivide_once(&self, radius: f32) -> Geometry {
        let mut points = Vec::with_capacity(self.points.len() * 2);
        let mut edges = Vec::with_capacity(self.edges.len() * 2);
        let mut faces = Vec::with_capacity(self.faces.len() * 4);

        let mut index = 0;

        for face in &self.faces {
            //
            //          p0
            //          /\
            //         /  \
            // p2_p0  /____\  p0_p1
            //       /\    /\
            //      /  \  /  \
            //     /____\/____\
            //   p2    p1_p2   p1
            //

            let p0    = math::project_to_radius(self.points[face[0]], radius);
            let p1    = math::project_to_radius(self.points[face[1]], radius);
            let p2    = math::project_to_radius(self.points[face[2]], radius);
            let p0_p1 = math::project_to_radius(math::midpoint(p0, p1), radius);
            let p1_p2 = math::project_to_radius(math::midpoint(p1, p2), radius);
            let p2_p0 = math::project_to_radius(math::midpoint(p2, p0), radius);

            points.push(p0);    let p0    = index; index += 1;
            points.push(p1);    let p1    = index; index += 1;
            points.push(p2);    let p2    = index; index += 1;
            points.push(p0_p1); let p0_p1 = index; index += 1;
            points.push(p1_p2); let p1_p2 = index; index += 1;
            points.push(p2_p0); let p2_p0 = index; index += 1;

            edges.push([p0,    p0_p1]);
            edges.push([p0,    p2_p0]);
            edges.push([p2_p0, p0_p1]);
            edges.push([p2_p0, p1_p2]);
            edges.push([p2_p0, p2]);
            edges.push([p2,    p1_p2]);
            edges.push([p0_p1, p1]);
            edges.push([p0_p1, p1_p2]);
            edges.push([p1_p2, p1]);

            faces.push([p0,    p0_p1, p2_p0]);
            faces.push([p0_p1, p1,    p1_p2]);
            faces.push([p2_p0, p1_p2, p2]);
            faces.push([p2_p0, p0_p1, p1_p2]);
        }

        Geometry {
            points: points,
            edges: edges,
            faces: faces,
        }
    }

    pub fn create_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::with_capacity(self.points.len() * 3);

        for face in &self.faces {
            let p0 = self.points[face[0]];
            let p1 = self.points[face[1]];
            let p2 = self.points[face[2]];

            let normal = math::face_normal(p0, p1, p2);

            vertices.push(Vertex { normal: normal.into(), position: p0.into() });
            vertices.push(Vertex { normal: normal.into(), position: p1.into() });
            vertices.push(Vertex { normal: normal.into(), position: p2.into() });
        }

        vertices
    }
}

fn create_camera(rotation: Rad<f32>, (width, height): (u32, u32)) -> Camera {
    Camera {
        position: Point3 {
            x: Rad::sin(rotation) * CAMERA_XZ_RADIUS,
            y: Rad::cos(rotation) * CAMERA_XZ_RADIUS,
            z: CAMERA_Y_HEIGHT,
        },
        target: Point3::origin(),
        projection: PerspectiveFov {
            aspect: width as f32 / height as f32,
            fovy: Rad::full_turn() / 6.0,
            near: CAMERA_NEAR,
            far: CAMERA_FAR,
        },
    }
}

fn draw_params<'a>(polygon_mode: PolygonMode) -> DrawParameters<'a> {
    DrawParameters {
        backface_culling: BackfaceCullingMode::CullClockwise,
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        polygon_mode: polygon_mode,
        ..DrawParameters::default()
    }
}

fn main() {
    let display = WindowBuilder::new()
        .with_title(WINDOW_TITLE.to_string())
        .with_dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let vertices = polyhedra::icosahedron()
        .subdivide(POLYHEDRON_RADIUS, POLYHEDRON_SUBDIVS)
        .create_vertices();

    let vertex_buffer = VertexBuffer::new(&display, &vertices).unwrap();
    let index_buffer = NoIndices(PrimitiveType::TrianglesList);

    let shaded_program =
        Program::from_source(&display,
                             include_str!("shader/shaded.v.glsl"),
                             include_str!("shader/shaded.f.glsl"),
                             None).unwrap();

    let flat_program =
        Program::from_source(&display,
                             include_str!("shader/flat.v.glsl"),
                             include_str!("shader/flat.f.glsl"),
                             None).unwrap();

    let mut camera_rotation = Rad::new(0.0);

    let mut show_mesh = true;

    'main: for time in times::in_seconds() {
        if let Some(window) = display.get_window() {
            window.set_title(&format!("{} | FPS: {:.2}",  WINDOW_TITLE, 1.0 / time.delta()));
        }

        let mut target = display.draw();

        target.clear_color_and_depth(color::DARK_GREY, 1.0);

        let rotation_delta = Rad::full_turn() * ROTATIONS_PER_SECOND * time.delta() as f32;
        camera_rotation = camera_rotation + rotation_delta;
        let camera = create_camera(camera_rotation, target.get_dimensions());
        let view_proj = camera.to_mat();

        let vector3_array: fn(Vector3<f32>) -> [f32; 3] = Vector3::into;
        let matrix4_array: fn(Matrix4<f32>) -> [[f32; 4]; 4] = Matrix4::into;

        target.draw(&vertex_buffer, &index_buffer, &shaded_program,
                    &uniform! {
                        color:      color::WHITE,
                        light_dir:  vector3_array(LIGHT_DIR),
                        model:      matrix4_array(Matrix4::identity()),
                        view_proj:  matrix4_array(view_proj),
                    },
                    &draw_params(PolygonMode::Fill)).unwrap();

        if show_mesh {
            target.draw(&vertex_buffer, &index_buffer, &flat_program,
                        &uniform! {
                            color:      color::BLACK,
                            // Scaled to prevent depth-fighting
                            model:      matrix4_array(Matrix4::from_scale(1.001)),
                            view_proj:  matrix4_array(view_proj),
                        },
                        &draw_params(PolygonMode::Line)).unwrap();
        }

        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                Event::Closed => break 'main,
                Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => match key {
                    Key::Escape => break 'main,
                    Key::M => show_mesh = !show_mesh,
                    _ => {},
                },
                _ => {},
            }
        }
    }
}
