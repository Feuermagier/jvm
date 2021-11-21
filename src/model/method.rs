use core::fmt::Debug;

use crate::interpreter::stack::StackValue;

use super::{types::JvmType, value::JvmValue, visibility::Visibility};

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub parameters: Vec<JvmType>,
    pub return_type: JvmType,
    pub visibility: Visibility,
    pub code: MethodCode,
    pub max_stack: usize,
    pub max_locals: usize,
}

#[derive(Debug)]
pub struct Parameters(Vec<StackValue>);

impl Parameters {
    pub fn of(values: Vec<StackValue>) -> Self {
        Self(values)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn to_vec(self) -> Vec<StackValue> {
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