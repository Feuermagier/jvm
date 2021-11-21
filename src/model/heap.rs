use super::class::{Class, Instance};

pub struct Heap {
    objects: Vec<Instance>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn resolve_mut(&mut self, index: HeapIndex) -> &mut Instance {
        &mut self.objects[index.0]
    }

    pub fn resolve(&self, index: HeapIndex) -> &Instance {
        &self.objects[index.0]
    }

    pub fn instantiate(&mut self, class: &Class) -> HeapIndex {
        let instance = class.instantiate();
        self.objects.push(instance);
        HeapIndex(self.objects.len() - 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct HeapIndex(usize);

impl HeapIndex {
    pub fn as_u16(self) -> u16 {
        self.0 as u16
    }

    pub fn from_u16(value: u16) -> Self {
        Self(value as usize)
    }
}
