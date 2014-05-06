extern crate serialize;

use self::serialize::{json, Encodable, Decodable};

use std::io;
use std::path;

pub struct ResourceManager {
    image_root:  Path,
    model_root:  Path,
    config_root: Path,
    script_root: Path
}

impl ResourceManager {
    pub fn init() -> ResourceManager {
        ResourceManager {
            image_root: Path::new(""),
            model_root: Path::new(""),
            config_root: Path::new(""),
            script_root: Path::new("")
        }
    }

    pub fn shutdown(&self) {
        //
    }
}
