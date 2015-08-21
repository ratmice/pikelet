
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io;
use std::env;

pub struct ResourceManager;

impl ResourceManager {

    pub fn path_for(&self, name: &str) -> PathBuf {
        env::current_dir()
            .map(|p| p.join("assets").join("name"))
            .unwrap_or(Path::new(".").to_path_buf())
    }

    fn open(&self, name: &str) -> io::Result<File> {
        let resolved_path = self.path_for(name);
        println!("DEBUG: Resource request for {}", resolved_path.display());
        File::open(&resolved_path)
    }
}
