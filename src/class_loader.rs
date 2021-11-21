use std::{fs::File, io::Read};

pub struct BootstrapClassLoader {}

impl BootstrapClassLoader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_class(&self, name: String) -> Vec<u8> {
        log::debug!("Loading class {}", name);
        let mut file = File::open(name + ".class").unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        bytes
    }
}
