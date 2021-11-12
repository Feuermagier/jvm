use core::fmt::Debug;

use super::{value::JvmValue, visibility::Visibility};

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub descriptor: String,
    pub visibility: Visibility,
    pub code: MethodCode,
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

pub enum MethodCode {
    Bytecode(Vec<u8>),
    Native(Option<Box<dyn Fn(Parameters) -> JvmValue>>)
}

impl Debug for MethodCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bytecode(arg0) => f.debug_tuple("Bytecode").field(arg0).finish(),
            Self::Native(arg0) => f.debug_tuple("Native").finish(),
        }
    }
}