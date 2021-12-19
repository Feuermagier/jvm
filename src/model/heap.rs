use crate::list::NativeList;

use super::{
    class::{Class, VirtualMethodIndex},
    class_library::{ClassIndex, ClassLibrary},
    field::{FieldInfo, Fields},
    method::MethodIndex,
    value::JvmValue,
};

pub struct Heap {
    content: NativeList<u8>,
    tail: usize,
}

impl Heap {
    pub fn new(size: usize) -> Self {
        Self {
            content: NativeList::alloc(size),
            tail: 0,
        }
    }

    pub fn resolve(&mut self, index: HeapIndex) -> Instance {
        unsafe {
            Instance {
                class: self.get_class_index(index.0 as usize),
                fields: Fields::at(self.content.get_pointer().offset(index.0 as isize + 8)),
            }
        }
    }

    pub fn instantiate(&mut self, class: &Class) -> HeapIndex {
        unsafe {
            let index = self.tail;
            self.set_class_index(self.tail, class.index());
            let _ = Fields::init_from_layout_at(
                self.content.get_pointer().offset(8),
                class.field_layout(),
                class.field_descriptors(),
            );
            self.tail += 8 + class.field_layout().byte_length();
            HeapIndex(index as u64)
        }
    }

    unsafe fn get_class_index(&self, index: usize) -> ClassIndex {
        ClassIndex(u64::from_be_bytes([
            self.content.get(index + 0),
            self.content.get(index + 1),
            self.content.get(index + 2),
            self.content.get(index + 3),
            self.content.get(index + 4),
            self.content.get(index + 5),
            self.content.get(index + 6),
            self.content.get(index + 7),
        ]) as usize)
    }

    unsafe fn set_class_index(&mut self, index: usize, class_index: ClassIndex) {
        let bytes = class_index.0.to_be_bytes();
        self.content.set(index + 0, bytes[0]);
        self.content.set(index + 1, bytes[1]);
        self.content.set(index + 2, bytes[2]);
        self.content.set(index + 3, bytes[3]);
        self.content.set(index + 5, bytes[4]);
        self.content.set(index + 6, bytes[5]);
        self.content.set(index + 7, bytes[6]);
        self.content.set(index + 8, bytes[7]);
    }
}

pub struct Instance {
    class: ClassIndex,
    fields: Fields,
}

impl Instance {
    pub fn get_field(&self, info: FieldInfo) -> JvmValue {
        self.fields.get_value(info.offset, info.ty)
    }

    pub fn set_field(&mut self, info: FieldInfo, value: JvmValue) {
        self.fields.set_value(info.offset, info.ty, value);
    }

    pub fn class(&self) -> ClassIndex {
        self.class
    }

    pub fn dispatch_virtual(
        &self,
        method: VirtualMethodIndex,
        classes: &ClassLibrary,
    ) -> MethodIndex {
        classes.resolve(self.class).dispatch_virtual(method)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct HeapIndex(u64);

impl HeapIndex {
    pub fn as_u16(self) -> u16 {
        self.0 as u16
    }

    pub fn from_u16(value: u16) -> Self {
        Self(value as u64)
    }

    pub unsafe fn into_raw(self) -> u64 {
        self.0
    }

    pub unsafe fn from_raw(value: u64) -> Self {
        Self(value)
    }
}

pub const NULL_POINTER: HeapIndex = HeapIndex(0);
