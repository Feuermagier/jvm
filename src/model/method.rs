use core::fmt::Debug;

use appendlist::AppendList;

use crate::interpreter::stack::StackValue;

use super::{
    class_library::ClassLibrary,
    heap::{Heap, HeapIndex},
    types::JvmType,
    value::JvmValue,
    visibility::Visibility,
};

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

#[derive(Debug, Clone)]
pub struct Method<'m> {
    pub name: &'m String,
    pub code: &'m Vec<u8>,
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

/*
pub enum MethodCode {
    Bytecode(Vec<u8>),
    Native(Box<dyn Fn(Parameters) -> JvmValue>),
}

impl Debug for MethodCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bytecode(arg0) => f.debug_tuple("Bytecode").field(arg0).finish(),
            Self::Native(_) => f.debug_tuple("Native").finish(),
        }
    }
}
*/

type MethodImplementation =
    dyn Fn(&mut Heap, &ClassLibrary, &MethodTable, Option<HeapIndex>, Parameters) -> JvmValue;

pub struct MethodTable {
    methods: AppendList<Box<MethodImplementation>>,
}

impl MethodTable {
    pub fn new() -> Self {
        Self {
            methods: AppendList::new(),
        }
    }

    pub fn add_method(&self, method: Box<MethodImplementation>) -> MethodIndex {
        self.methods.push(method);
        MethodIndex(self.methods.len() - 1)
    }

    pub fn resolve(&self, method_index: MethodIndex) -> &MethodImplementation {
        &self.methods[method_index.0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct MethodIndex(usize);
