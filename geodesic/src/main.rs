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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct NodeIndex(pub usize);
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct EdgeIndex(pub usize);
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct FaceIndex(pub usize);

#[derive(Clone, Debug)]
pub struct Node {
    position: Point3<f32>,
}

pub type Edge = [NodeIndex; 2];
pub type Face = [NodeIndex; 3];

#[derive(Clone, Debug)]
pub struct Geometry {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub faces: Vec<Face>,
}

impl Geometry {
    pub fn node(&self, index: NodeIndex) -> &Node {
        &self.nodes[index.0]
    }

    pub fn edge(&self, index: EdgeIndex) -> &Edge {
        &self.edges[index.0]
    }

    pub fn face(&self, index: FaceIndex) -> &Face {
        &self.faces[index.0]
    }

    pub fn subdivide(&self, radius: f32, count: usize) -> Geometry {
        (0..count).fold(self.clone(), |acc, _| acc.subdivide_once(radius))
    }

    pub fn subdivide_once(&self, radius: f32) -> Geometry {
        let mut nodes = Vec::with_capacity(self.nodes.len() * 2);
        let mut edges = Vec::with_capacity(self.edges.len() * 2);
        let mut faces = Vec::with_capacity(self.faces.len() * 4);

        let push_node = |nodes: &mut Vec<_>, node| {
            nodes.push(node);
            NodeIndex(nodes.len() - 1)
        };

        for face in &self.faces {
            //
            //          n0
            //          /\
            //         /  \
            // n2_n0  /____\  n0_n1
            //       /\    /\
            //      /  \  /  \
            //     /____\/____\
            //   n2    n1_n2   n1
            //

            let p0    = math::set_radius(self.node(face[0]).position, radius);
            let p1    = math::set_radius(self.node(face[1]).position, radius);
            let p2    = math::set_radius(self.node(face[2]).position, radius);
            let p0_p1 = math::set_radius(math::midpoint(p0, p1), radius);
            let p1_p2 = math::set_radius(math::midpoint(p1, p2), radius);
            let p2_p0 = math::set_radius(math::midpoint(p2, p0), radius);

            let n0    = push_node(&mut nodes, Node { position: p0 });
            let n1    = push_node(&mut nodes, Node { position: p1 });
            let n2    = push_node(&mut nodes, Node { position: p2 });
            let n0_n1 = push_node(&mut nodes, Node { position: p0_p1 });
            let n1_n2 = push_node(&mut nodes, Node { position: p1_p2 });
            let n2_n0 = push_node(&mut nodes, Node { position: p2_p0 });

            edges.push([n0,    n0_n1]);
            edges.push([n0,    n2_n0]);
            edges.push([n2_n0, n0_n1]);
            edges.push([n2_n0, n1_n2]);
            edges.push([n2_n0, n2]);
            edges.push([n2,    n1_n2]);
            edges.push([n0_n1, n1]);
            edges.push([n0_n1, n1_n2]);
            edges.push([n1_n2, n1]);

            faces.push([n0,    n0_n1, n2_n0]);
            faces.push([n0_n1, n1,    n1_n2]);
            faces.push([n2_n0, n1_n2, n2]);
            faces.push([n2_n0, n0_n1, n1_n2]);
        }

        Geometry {
            nodes: nodes,
            edges: edges,
            faces: faces,
        }
    }

    pub fn create_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::with_capacity(self.nodes.len() * 3);

        for face in &self.faces {
            let p0 = self.node(face[0]).position;
            let p1 = self.node(face[1]).position;
            let p2 = self.node(face[2]).position;

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
    let mut is_rotating = true;

    'main: for time in times::in_seconds() {
        if let Some(window) = display.get_window() {
            window.set_title(&format!("{} | FPS: {:.2}",  WINDOW_TITLE, 1.0 / time.delta()));
        }

        let mut target = display.draw();

        if is_rotating {
            let delta = Rad::full_turn() * ROTATIONS_PER_SECOND * time.delta() as f32;
            camera_rotation = camera_rotation + delta;
        }
        let camera = create_camera(camera_rotation, target.get_dimensions());
        let view_proj = camera.to_mat();

        target.clear_color_and_depth(color::DARK_GREY, 1.0);

        target.draw(&vertex_buffer, &index_buffer, &shaded_program,
                    &uniform! {
                        color:      color::WHITE,
                        light_dir:  math::array_v3(LIGHT_DIR),
                        model:      math::array_m4(Matrix4::identity()),
                        view_proj:  math::array_m4(view_proj),
                    },
                    &draw_params(PolygonMode::Fill)).unwrap();

        if show_mesh {
            target.draw(&vertex_buffer, &index_buffer, &flat_program,
                        &uniform! {
                            color:      color::BLACK,
                            // Scaled to prevent depth-fighting
                            model:      math::array_m4(Matrix4::from_scale(1.001)),
                            view_proj:  math::array_m4(view_proj),
                        },
                        &draw_params(PolygonMode::Line)).unwrap();
        }

        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                Event::Closed => break 'main,
                Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => match key {
                    Key::M => show_mesh = !show_mesh,
                    Key::Space => is_rotating = !is_rotating,
                    Key::Escape => break 'main,
                    _ => {},
                },
                _ => {},
            }
        }
    }
}
