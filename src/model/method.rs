use core::fmt::Debug;
use std::{cell::{RefCell, Ref}, borrow::Borrow};

use appendlist::AppendList;

use crate::{
    interpreter::{self},
    jit::CodeBuffer,
    list::NativeList,
};

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
    pub is_virtual: bool,
}

impl MethodDescriptor {
    pub fn parameter_count(&self) -> usize {
        self.parameters.iter().map(|p| p.size()).sum::<usize>() / 4
            + if self.is_virtual { 1 } else { 0 }
    }
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

pub type NativeMethod = extern "sysv64" fn(
    MethodIndex,
    StackPointer,
    *mut Heap,
    *const ClassLibrary,
    *const MethodTable,
) -> JvmValue;

pub enum MethodImplementation {
    Native(Box<NativeMethod>, Box<dyn CodeBuffer>),
    Interpreted,
}

#[repr(C)]
pub struct MethodTable {
    call_table: NativeList<u64>,
    methods: RefCell<Vec<MethodEntry>>,
}

impl MethodTable {
    pub fn new(length: usize) -> Self {
        Self {
            call_table: NativeList::alloc(length, 8),
            methods: RefCell::new(Vec::new()),
        }
    }

    pub fn add_method(
        &self,
        implementation: MethodImplementation,
        data: MethodData,
    ) -> MethodIndex {
        let index = self.methods.borrow().len();
        self.patch_call_table(index, &implementation);
        self.methods.borrow_mut().push(MethodEntry {
            implementation,
            data,
        });
        index.into()
    }

    pub fn update_method(&self, index: MethodIndex, implementation: MethodImplementation) {
        self.patch_call_table(index.into(), &implementation);
        self.methods.borrow_mut()[index.0 as usize].implementation = implementation;
    }

    pub unsafe fn resolve(&self, method_index: MethodIndex) -> u64 {
        self.call_table.get(method_index.0 as usize)
    }

    pub unsafe fn call_table_pointer(&self) -> *mut u64 {
        self.call_table.get_pointer()
    }

    pub fn get_data(&self, method_index: MethodIndex) -> Ref<'_, MethodData> {
        Ref::map(self.methods.borrow(), |m| &m[method_index.0 as usize].data)
    }

    pub fn method_count(&self) -> usize {
        self.methods.borrow().len()
    }

    fn patch_call_table(&self, index: usize, implementation: &MethodImplementation) {
        let ptr = match &implementation {
            MethodImplementation::Native(code, _) => **code as u64,
            MethodImplementation::Interpreted => interpreter::interpreter_trampoline as u64,
        };
        unsafe {
            self.call_table.set(index, ptr);
        }
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

impl MethodData {
    pub fn from_bytecode_descriptor(
        desc: &MethodDescriptor,
        owning_class: ClassIndex,
    ) -> Option<Self> {
        if let MethodCode::Bytecode(code) = &desc.code {
            // +1 for this
            let parameter_count = desc.parameter_count();

            Some(Self {
                name: desc.name.clone(),
                code: code.clone(),
                max_stack: desc.max_stack,
                max_locals: desc.max_locals,
                owning_class,
                argument_count: parameter_count,
                return_type: desc.return_type,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct MethodIndex(u32);

impl MethodIndex {
    pub unsafe fn into_raw(self) -> u32 {
        self.0
    }

    pub unsafe fn from_raw(value: u32) -> Self {
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
        MethodIndex(index as u32)
    }
}
