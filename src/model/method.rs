use core::fmt::Debug;

use crate::interpreter::stack::StackValue;

use super::{types::JvmType, value::JvmValue, visibility::Visibility};

#[derive(Debug)]
pub struct MethodDescriptor {
    pub name: String,
    pub parameters: Vec<JvmType>,
    pub return_type: JvmType,
    pub visibility: Visibility,
    pub code: Option<Vec<u8>>,
    pub max_stack: usize,
    pub max_locals: usize,
}

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

impl Method {
    pub fn new_bytecode_method(descriptor: &MethodDescriptor) -> Self {
        Self {
            name: descriptor.name.clone(),
            parameters: descriptor.parameters.clone(),
            return_type: descriptor.return_type,
            visibility: descriptor.visibility,
            code: MethodCode::Bytecode(
                descriptor
                    .code
                    .clone()
                    .expect("This is not a bytecode method!"),
            ),
            max_stack: descriptor.max_stack,
            max_locals: descriptor.max_locals,
        }
    }

    pub fn new_native_method(
        descriptor: &MethodDescriptor,
        implementation: Box<dyn Fn(Parameters) -> JvmValue>,
    ) -> Self {
        Self {
            name: descriptor.name.clone(),
            parameters: descriptor.parameters.clone(),
            return_type: descriptor.return_type,
            visibility: descriptor.visibility,
            code: MethodCode::Native(implementation),
            max_stack: descriptor.max_stack,
            max_locals: descriptor.max_locals,
        }
    }
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
    Native(Box<dyn Fn(Parameters) -> JvmValue>),
}

impl Debug for MethodCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bytecode(arg0) => f.debug_tuple("Bytecode").field(arg0).finish(),
            Self::Native(arg0) => f.debug_tuple("Native").finish(),
        }
    }
}
