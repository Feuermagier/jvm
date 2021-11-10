#[derive(Debug)]
pub struct ClassFile {
    minor_version: u16,
    major_version: u16,
}

impl ClassFile {
    pub fn new(minor_version: u16, major_version: u16) -> Self {
        Self {
            minor_version,
            major_version,
        }
    }

    pub fn minor_version(&self) -> u16 {
        self.minor_version
    }

    pub fn major_version(&self) -> u16 {
        self.major_version
    }
}
