use std::fmt::Debug;

use crate::model::value::JvmValue;

pub struct InterpreterStack {
    stack: Vec<JvmValue>,
}

impl InterpreterStack {
    pub fn new(capacity: usize) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn push(&mut self, value: JvmValue) {
        self.stack.push(value);
    }

    #[inline]
    pub fn pop(&mut self) -> JvmValue {
        self.stack
            .pop()
            .expect("Trying to pop from an empty local stack")
    }
}

impl Debug for InterpreterStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Local interpreter stack\n")?;
        for value in &mut self.stack.iter().rev() {
            write!(f, "\t{}\n", value)?;
        }
        Ok(())
    }
}
