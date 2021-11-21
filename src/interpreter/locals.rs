use crate::model::{heap::HeapIndex, method::Parameters, value::JvmReference};

use super::stack::StackValue;

pub struct InterpreterLocals {
    locals: Vec<StackValue>,
}

impl InterpreterLocals {
    pub fn new(capacity: usize, parameters: Parameters, this: Option<HeapIndex>) -> Self {
        let mut locals = if let Some(this) = this {
            let mut locals = vec![StackValue::from_reference(JvmReference::from_heap_index(this))];
            locals.extend(parameters.to_vec());
            locals
        } else {
            parameters.to_vec()
        };
        let locals_count = capacity - locals.len();
        locals.reserve_exact(locals_count);

        for _ in 0..locals_count {
            locals.push(StackValue::default());
        }

        Self { locals }
    }

    pub fn get(&self, index: usize) -> StackValue {
        self.locals[index]
    }

    pub fn set(&mut self, index: usize, value: StackValue) {
        self.locals[index] = value;
    }
}
