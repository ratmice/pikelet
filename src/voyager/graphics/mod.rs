extern crate gl;
extern crate native;
extern crate cgmath;

use platform::Platform;

use resources::ResourceManager;
use data::Handle;
use data::EngineData;

use self::shader_program::ShaderProgram;
use self::buffers::VertexBuffer;
use self::materials::Material;

mod shader_program;
mod materials;
mod buffers;

trait Bind {
    fn bind();
}

pub type ShaderProgramHandle = Handle;
pub type VertexBufferHandle = Handle;
pub type MaterialHandle = Handle;

/// A graphics device manager
pub struct GraphicsManager {
    vertex_buffers: EngineData<VertexBuffer>,
    shader_programs: EngineData<ShaderProgram>,
    materials: EngineData<Material>
}

impl GraphicsManager {
    /// Initialise a new graphics device manager
    pub fn init<T: Platform>(platform: &T, resources: &ResourceManager) -> GraphicsManager {
        let vertex_buffers = EngineData::new();
        let shader_programs = EngineData::new();
        let materials = EngineData::new();
        
        platform.load_gl(gl::load_with);
        
        gl::ClearColor(0.3,0.3,0.3,1.0);
        
        let mut manager = GraphicsManager {
            vertex_buffers: vertex_buffers,
            shader_programs: shader_programs,
            materials: materials
        };

        let material_config = resources.open_config("materials.json")
            .expect("Failed to load material configuration!");
        println!("DEBUG: materials configuration: {}", material_config.to_pretty_str());

        //// load shader programs

        let shader_program_defs = material_config.find(&StrBuf::from_str("programs"))
            .and_then(|c| c.as_object())
            .expect("ERROR: Unable to find programs section in materials config.");
        
        for (program_name, program_config) in shader_program_defs.iter() {
            let vertex_src = program_config.find(&StrBuf::from_str("vertex"))
                .and_then(|v| v.as_string())
                .and_then(|p| {
                    resources.open_shader(p)
                })
                .expect("ERROR: Unable to read vertex shader!");

            let fragment_src = program_config.find(&StrBuf::from_str("fragment"))
                .and_then(|v| v.as_string())
                .and_then(|p| {
                    resources.open_shader(p)
                })
                .expect("ERROR: Unable to read fragment shader!");

            let handle = manager.add_shader_program(vertex_src, fragment_src);
        }

        //// load materials

        let material_defs = material_config.find(&StrBuf::from_str("materials"))
            .expect("ERROR: Unable to find materials section in materials config.");

        manager
    }

    pub fn shutdown(&mut self) {
        self.vertex_buffers.clear();
        self.shader_programs.clear();
    }

    // TODO this might be renamed
    pub fn clear(&self) {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    fn add_vertex_buffer<T>(&mut self, data: ~[f32], stride: u32) -> Option<VertexBufferHandle> {
        VertexBuffer::new(data, stride)
            .and_then(|buffer| {
                Some(self.vertex_buffers.add(buffer))
            })
    }

    fn destroy_vertex_buffer(&mut self, handle: Handle) {
        self.vertex_buffers.remove(handle);
    }

    fn add_shader_program(&mut self, vert_src: &str, frag_src: &str) -> Option<ShaderProgramHandle> {
        ShaderProgram::new(vert_src, frag_src)
            .and_then(|program| {
                Some(self.shader_programs.add(program))
            })
    }

    fn destroy_shader_program(&mut self, handle: Handle) {
        self.shader_programs.remove(handle);
    }

    pub fn cube(width: uint, height: uint, depth: uint) -> Option<VertexBufferHandle> {
        None
    }

    pub fn patch(width: uint, height: uint) -> Option<VertexBufferHandle> {
        None
    }
}

impl Drop for GraphicsManager {
    fn drop(&mut self) {
        // Clean up all the things
        self.shutdown()
    }
}
