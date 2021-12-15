use core::fmt::Debug;

use appendlist::AppendList;

use crate::interpreter::stack::StackValue;

use super::{
    class_library::{ClassLibrary, ClassIndex},
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
    pub code: MethodCode,
    pub max_stack: usize,
    pub max_locals: usize,
}

/*
#[derive(Debug, Clone)]
pub struct Method<'m> {
    pub name: &'m String,
    pub code: &'m Vec<u8>,
    pub max_stack: usize,
    pub max_locals: usize,
}
*/

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
    Native,
    Abstract
}

impl Debug for MethodCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bytecode(arg0) => f.debug_tuple("Bytecode").field(arg0).finish(),
            Self::Native => f.debug_tuple("Native").finish(),
            Self::Abstract => f.debug_tuple("Abstract").finish(),
        }
    }
}

pub type MethodImplementation =
    extern "sysv64" fn(MethodIndex, &mut Heap, &ClassLibrary, &MethodTable, Option<HeapIndex>, Parameters) -> JvmValue;

pub struct MethodTable {
    methods: AppendList<MethodEntry>,
}

impl MethodTable {
    pub fn new() -> Self {
        Self {
            methods: AppendList::new(),
        }
    }

    pub fn add_method(&self, implementation: Box<MethodImplementation>, data: MethodData) -> MethodIndex {
        self.methods.push(MethodEntry { implementation, data });
        (self.methods.len() - 1).into()
    }

    pub fn resolve(&self, method_index: MethodIndex) -> &MethodImplementation {
        &self.methods[method_index.into()].implementation
    }

    pub fn get_data(&self, method_index: MethodIndex) -> &MethodData {
        &self.methods[method_index.into()].data
    }
}

#[repr(C)]
pub struct MethodEntry {
    pub implementation: Box<MethodImplementation>,
    pub data: MethodData
}

pub struct MethodData {
    pub name: String,
    pub code: Vec<u8>,
    pub max_stack: usize,
    pub max_locals: usize,
    pub owning_class: ClassIndex
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct MethodIndex(u64);

impl From<MethodIndex> for usize {
    fn from(index: MethodIndex) -> Self {
        index.0 as usize
    }
}

impl From<usize> for MethodIndex {
    fn from(index: usize) -> Self {
        MethodIndex(index as u64)
    }
}