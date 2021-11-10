use super::{value::JvmValue, visibility::Visibility};

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub descriptor: String,
    pub visibility: Visibility,
    pub code: Vec<u8>,
    pub max_stack: usize,
    pub max_locals: usize,
}

pub struct Parameters(Vec<JvmValue>);

impl Parameters {
    pub fn of(values: Vec<JvmValue>) -> Self {
        Self(values)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn to_vec(self) -> Vec<JvmValue> {
        self.0
    }
}