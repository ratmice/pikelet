extern crate cgmath;
#[macro_use] extern crate glium;
extern crate time;

pub use glium::glutin;

use cgmath::{Angle, PerspectiveFov, Rad};
use cgmath::{Matrix4, SquareMatrix};
use cgmath::{Point3, Point, Vector3};
use glium::{DisplayBuild, DrawParameters, PolygonMode, Program, Surface, VertexBuffer};
use glium::index::{PrimitiveType, NoIndices};
use glutin::{ElementState, Event, WindowBuilder};
use glutin::VirtualKeyCode as Key;
use std::ops::{Index, IndexMut};

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

pub trait Indexer {
    type Element;

    fn index(self) -> usize;
}

pub fn get<I: Indexer, Elements>(elements: &Elements, index: I) -> &I::Element where
    Elements: Index<usize, Output = I::Element>,
{
    &elements[index.index()]
}

pub fn get_mut<I: Indexer, Elements>(elements: &mut Elements, index: I) -> &mut I::Element where
    Elements: IndexMut<usize, Output = I::Element>,
{
    &mut elements[index.index()]
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct NodeIndex(pub usize);
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct EdgeIndex(pub usize);
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct FaceIndex(pub usize);

impl Indexer for NodeIndex { type Element = Node; fn index(self) -> usize { self.0 } }
impl Indexer for EdgeIndex { type Element = Edge; fn index(self) -> usize { self.0 } }
impl Indexer for FaceIndex { type Element = Face; fn index(self) -> usize { self.0 } }

#[derive(Clone, Debug)]
pub struct Node {
    pub position: Point3<f32>,
    pub edges: Vec<EdgeIndex>,
    pub faces: Vec<FaceIndex>,

    // Pentagon
    // pub edges: [EdgeIndex; 5],
    // pub faces: [FaceIndex; 5],

    // Hexagon
    // pub edges: [EdgeIndex; 6],
    // pub faces: [FaceIndex; 6],
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub nodes: [NodeIndex; 2],
    pub faces: Vec<FaceIndex>,
    // pub faces: [EdgeIndex; 2],
}

#[derive(Clone, Debug)]
pub struct Face {
    pub nodes: [NodeIndex; 3],
    pub edges: Vec<EdgeIndex>,
    // pub edges: [EdgeIndex; 3],
}

#[derive(Clone, Debug)]
pub struct Geometry {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub faces: Vec<Face>,
}

impl Geometry {
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

        let push_edge = |edges: &mut Vec<_>, edge| {
            edges.push(edge);
            EdgeIndex(edges.len() - 1)
        };

        for face in &self.faces {
            //
            //          n0
            //          /\
            //         /  \
            //    n5  /____\  n3
            //       /\    /\
            //      /  \  /  \
            //     /____\/____\
            //   n2     n4     n1
            //

            let p0 = math::set_radius(get(&self.nodes, face.nodes[0]).position, radius);
            let p1 = math::set_radius(get(&self.nodes, face.nodes[1]).position, radius);
            let p2 = math::set_radius(get(&self.nodes, face.nodes[2]).position, radius);
            let p3 = math::set_radius(math::midpoint(p0, p1), radius);
            let p4 = math::set_radius(math::midpoint(p1, p2), radius);
            let p5 = math::set_radius(math::midpoint(p2, p0), radius);

            let n0 = push_node(&mut nodes, Node { position: p0, edges: vec![], faces: vec![] });
            let n1 = push_node(&mut nodes, Node { position: p1, edges: vec![], faces: vec![] });
            let n2 = push_node(&mut nodes, Node { position: p2, edges: vec![], faces: vec![] });
            let n3 = push_node(&mut nodes, Node { position: p3, edges: vec![], faces: vec![] });
            let n4 = push_node(&mut nodes, Node { position: p4, edges: vec![], faces: vec![] });
            let n5 = push_node(&mut nodes, Node { position: p5, edges: vec![], faces: vec![] });

            let n0_n3 = push_edge(&mut edges, Edge { nodes: [n0, n3], faces: vec![] });
            let n0_n5 = push_edge(&mut edges, Edge { nodes: [n0, n5], faces: vec![] });
            let n5_n3 = push_edge(&mut edges, Edge { nodes: [n5, n3], faces: vec![] });
            let n5_n4 = push_edge(&mut edges, Edge { nodes: [n5, n4], faces: vec![] });
            let n5_n2 = push_edge(&mut edges, Edge { nodes: [n5, n2], faces: vec![] });
            let n2_n4 = push_edge(&mut edges, Edge { nodes: [n2, n4], faces: vec![] });
            let n3_n1 = push_edge(&mut edges, Edge { nodes: [n3, n1], faces: vec![] });
            let n3_n4 = push_edge(&mut edges, Edge { nodes: [n3, n4], faces: vec![] });
            let n4_n1 = push_edge(&mut edges, Edge { nodes: [n4, n1], faces: vec![] });

            faces.push(Face { nodes: [n0, n3, n5], edges: vec![n0_n3, n0_n5, n5_n3] });
            faces.push(Face { nodes: [n3, n1, n4], edges: vec![n3_n1, n3_n4, n4_n1] });
            faces.push(Face { nodes: [n5, n4, n2], edges: vec![n5_n4, n5_n2, n2_n4] });
            faces.push(Face { nodes: [n5, n3, n4], edges: vec![n5_n3, n5_n4, n3_n4] });
        }

        for (index, edge) in edges.iter().enumerate() {
            get_mut(&mut nodes, edge.nodes[0]).edges.push(EdgeIndex(index));
            get_mut(&mut nodes, edge.nodes[1]).edges.push(EdgeIndex(index));
        }

        for (index, face) in faces.iter().enumerate() {
            get_mut(&mut nodes, face.nodes[0]).faces.push(FaceIndex(index));
            get_mut(&mut nodes, face.nodes[1]).faces.push(FaceIndex(index));
            get_mut(&mut nodes, face.nodes[2]).faces.push(FaceIndex(index));
        }

        for (index, face) in faces.iter().enumerate() {
            get_mut(&mut edges, face.edges[0]).faces.push(FaceIndex(index));
            get_mut(&mut edges, face.edges[1]).faces.push(FaceIndex(index));
            get_mut(&mut edges, face.edges[2]).faces.push(FaceIndex(index));
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
            let n0 = get(&self.nodes, face.nodes[0]).position;
            let n1 = get(&self.nodes, face.nodes[1]).position;
            let n2 = get(&self.nodes, face.nodes[2]).position;

            let normal = math::face_normal(n0, n1, n2);

            vertices.push(Vertex { normal: normal.into(), position: n0.into() });
            vertices.push(Vertex { normal: normal.into(), position: n1.into() });
            vertices.push(Vertex { normal: normal.into(), position: n2.into() });
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
    use glium::{BackfaceCullingMode, Depth, DepthTest};

    DrawParameters {
        backface_culling: BackfaceCullingMode::CullClockwise,
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Depth::default()
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
