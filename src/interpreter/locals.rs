use std::vec;

use crate::model::{class::Instance, method::Parameters, value::JvmValue};

pub struct InterpreterLocals {
    locals: Vec<JvmValue>,
    this: Option<JvmValue>,
}

impl InterpreterLocals {
    pub fn new(capacity: usize, parameters: Parameters, this: Option<JvmValue>) -> Self {
        let mut locals = parameters.to_vec();
        let locals_count = capacity - locals.len();
        locals.reserve_exact(locals_count);

        for _ in 0..locals_count {
            locals.push(JvmValue::Void);
        }

        Self {
            locals: locals,
            this,
        }
    }

    pub fn get(&self, index: usize) -> JvmValue {
        if let Some(this) = self.this {
            if index == 0 {
                this
            } else {
                self.locals[index - 1]
            }
        } else {
            self.locals[index]
        }
    }

    pub fn iget(&self, index: usize) -> i32 {
        let value = self.locals[index];
        match value {
            JvmValue::Int(value) => value,
            _ => panic!(
                "Expected an int value in local variable #{}, but got a {}",
                index, value
            ),
        }
    }

    pub fn set(&mut self, index: usize, value: JvmValue) {
        self.locals[index] = value;
    }

    pub fn iset(&mut self, index: usize, value: i32) {
        self.locals[index] = JvmValue::Int(value);
    }
}
