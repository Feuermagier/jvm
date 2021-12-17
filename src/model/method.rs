use core::fmt::Debug;
use std::alloc::Layout;

use appendlist::AppendList;

use crate::interpreter::{self};

use super::{
    class_library::{ClassIndex, ClassLibrary},
    heap::Heap,
    stack::StackPointer,
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

pub enum MethodCode {
    Bytecode(Vec<u8>),
    Native,
    Abstract,
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

pub enum MethodImplementation {
    Native(
        Box<
            extern "sysv64" fn(
                MethodIndex,
                StackPointer,
                *mut Heap,
                *const ClassLibrary,
                *const MethodTable,
            ) -> JvmValue,
        >,
    ),
    Interpreted,
}

#[repr(C)]
pub struct MethodTable {
    call_table: *mut u64,
    methods: AppendList<MethodEntry>,
}

impl MethodTable {
    pub fn new(capacity: usize) -> Self {
        let call_table_layout = Layout::from_size_align(capacity, 8).unwrap();
        let call_table = unsafe { std::alloc::alloc(call_table_layout) as *mut u64 };
        Self {
            call_table,
            methods: AppendList::new(),
        }
    }

    pub fn add_method(
        &self,
        implementation: MethodImplementation,
        data: MethodData,
    ) -> MethodIndex {
        let index = self.methods.len();
        let ptr = match &implementation {
            MethodImplementation::Native(code) => **code as u64,
            MethodImplementation::Interpreted => interpreter::interpreter_trampoline as u64,
        };
        unsafe {
            *self.call_table.offset(index as isize) = ptr;
        }
        self.methods.push(MethodEntry {
            implementation,
            data,
        });
        (self.methods.len() - 1).into()
    }

    pub unsafe fn resolve(&self, method_index: MethodIndex) -> u64 {
        *self.call_table.offset(method_index.0 as isize)
    }

    pub fn get_data(&self, method_index: MethodIndex) -> &MethodData {
        &self.methods[method_index.into()].data
    }

    pub fn method_count(&self) -> usize {
        self.methods.len()
    }
}

#[repr(C)]
pub struct MethodEntry {
    pub implementation: MethodImplementation,
    pub data: MethodData,
}

pub struct MethodData {
    pub name: String,
    pub code: Vec<u8>,
    pub max_stack: usize,
    pub max_locals: usize,
    pub owning_class: ClassIndex,
    pub argument_count: usize,
    pub return_type: JvmType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct MethodIndex(u64);

impl MethodIndex {
    pub unsafe fn into_raw(self) -> u64 {
        self.0
    }

    pub unsafe fn from_raw(value: u64) -> Self {
        Self(value)
    }
}

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
