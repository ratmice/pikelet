
extern crate serialize;

use self::serialize::json;

use std::io::{File, FileMode, FileAccess, IoResult};
use std::io::{Open, Read};
use std::os;

pub enum ResourceType {
    Config,
    Script,
    Image,
    Model,
    Audio,
    Shader
}

pub struct ResourceManager {
    image_root:  Path,
    model_root:  Path,
    audio_root:  Path,
    config_root: Path,
    script_root: Path,
    shader_root: Path
}

impl ResourceManager {
    pub fn init() -> ResourceManager {
        let root_dir = os::self_exe_path().unwrap_or(Path::new("."));
        println!("INFO: Application root directory: {}", root_dir.display());

        ResourceManager {
            image_root: root_dir.join("assets/images"),
            model_root: root_dir.join("assets/models"),
            audio_root: root_dir.join("assets/audio"),
            config_root: root_dir.join("assets/conf"),
            script_root: root_dir.join("assets/scripts"),
            shader_root: root_dir.join("assets/shaders")
        }
    }

    pub fn shutdown(&self) {
        //
    }

    pub fn path_for(&self, name: &str, res_type: ResourceType) -> Path {
        let resource_root: &Path = match res_type {
            Config => &self.config_root,
            Script => &self.script_root,
            Image => &self.image_root,
            Model => &self.model_root,
            Audio => &self.audio_root,
            Shader => &self.shader_root
        };
        resource_root.join(name)
    }

    fn open(&self, name: &str, res_type: ResourceType, mode: FileMode, access: FileAccess) -> IoResult<File> {
        let resolved_path = self.path_for(name, res_type);
        println!("DEBUG: Resource request for {}", resolved_path.display());
        File::open_mode(&resolved_path, mode, access)
    }

    pub fn open_config(&self, name: &str) -> Option<json::Json> {
        match self.open(name, Config, Open, Read)
            .and_then(|mut f| f.read_to_str()) {
                Ok(s) => match json::from_str(s.as_slice()) {
                    Ok(object) => Some(object),
                    Err(err) => {
                        println!("ERROR: While parsing configuration: {}", err);
                        None
                    }
                },
                Err(err) => {
                    println!("ERROR: While reading configuration: {}", err);
                    None
                }
            }
    }

    pub fn open_shader(&self, name: &str) -> Option<String> {
        match self.open(name, Shader, Open, Read)
            .and_then(|mut f| f.read_to_str()) {
                Ok(s) => Some(s),
                Err(err) => {
                    println!("ERROR: While reading shader: {}", err);
                    None
                }
            }
    }
}
