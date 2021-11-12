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

    pub fn resolve(&mut self, index: HeapIndex) -> &mut Instance {
        &mut self.objects[index.0]
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
