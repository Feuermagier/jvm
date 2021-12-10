use std::fmt::Debug;

use crate::model::{
    method::Parameters,
    types::JvmType,
    value::{JvmDouble, JvmFloat, JvmInt, JvmLong, JvmReference, JvmValue},
};

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy)]
pub struct StackValue(u32);

impl StackValue {
    pub fn as_int(self) -> JvmInt {
        unsafe { std::mem::transmute::<u32, i32>(self.0) }.into()
    }

    pub fn as_float(self) -> JvmFloat {
        unsafe { std::mem::transmute::<u32, f32>(self.0) }.into()
    }

    pub fn as_reference(self) -> JvmReference {
        (self.0 as u16).into()
    }

    pub fn from_int(value: JvmInt) -> Self {
        Self(unsafe { std::mem::transmute(value.0) })
    }

    pub fn from_float(value: JvmFloat) -> Self {
        Self(unsafe { std::mem::transmute(value.0) })
    }

    pub fn from_long(value: JvmLong) -> (Self, Self) {
        let value: u64 = unsafe { std::mem::transmute(value.0) };
        let high = ((value & 0xffffffff00000000) >> 32) as u32;
        let low = (value & 0xffffffff) as u32;
        (Self(high), Self(low))
    }

    pub fn from_double(value: JvmDouble) -> (Self, Self) {
        let value: u64 = unsafe { std::mem::transmute(value.0) };
        let high = ((value & 0xffffffff00000000) >> 32) as u32;
        let low = (value & 0xffffffff) as u32;
        (Self(high), Self(low))
    }

    pub fn from_reference(value: JvmReference) -> Self {
        Self(value.0 as u32)
    }
}

pub trait StackValueWide {
    fn as_long(self) -> JvmLong;
    fn as_double(self) -> JvmDouble;
}

impl StackValueWide for (StackValue, StackValue) {
    fn as_long(self) -> JvmLong {
        unsafe { std::mem::transmute::<u64, i64>((self.0 .0 as u64) << 32 | (self.1 .0 as u64)) }
            .into()
    }

    fn as_double(self) -> JvmDouble {
        unsafe { std::mem::transmute::<u64, f64>((self.0 .0 as u64) << 32 | (self.1 .0 as u64)) }
            .into()
    }
}

pub struct InterpreterStack {
    stack: Vec<StackValue>,
}

impl InterpreterStack {
    pub fn new(capacity: usize) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn push(&mut self, value: StackValue) {
        self.stack.push(value);
    }

    #[inline]
    pub fn push_wide(&mut self, values: (StackValue, StackValue)) {
        self.stack.push(values.0);
        self.stack.push(values.1);
    }

    #[inline]
    pub fn pop(&mut self) -> StackValue {
        self.stack
            .pop()
            .expect("Trying to pop from an empty local stack")
    }

    #[inline]
    pub fn pop_type(&mut self, ty: JvmType) -> JvmValue {
        match ty {
            JvmType::Void => JvmValue::Void,
            JvmType::Byte => todo!(),
            JvmType::Char => todo!(),
            JvmType::Integer => JvmValue::Int(self.pop().as_int()),
            JvmType::Long => JvmValue::Long(self.pop_wide().as_long()),
            JvmType::Float => JvmValue::Float(self.pop().as_float()),
            JvmType::Double => JvmValue::Double(self.pop_wide().as_double()),
            JvmType::Reference => JvmValue::Reference(self.pop().as_reference()),
            JvmType::Short => todo!(),
            JvmType::Boolean => todo!(),
        }
    }

    #[inline]
    pub fn pop_wide(&mut self) -> (StackValue, StackValue) {
        let top = self.pop();
        let second = self.pop();
        (second, top)
    }

    #[inline]
    pub fn pop_parameters(&mut self, count: usize) -> Parameters {
        Parameters::of(self.stack.drain(self.stack.len() - count..).collect())
    }

    pub fn push_value(&mut self, value: JvmValue) {
        match value {
            JvmValue::Void => {}
            JvmValue::Int(value) => self.push(StackValue::from_int(value)),
            JvmValue::Long(value) => self.push_wide(StackValue::from_long(value)),
            JvmValue::Float(value) => self.push(StackValue::from_float(value)),
            JvmValue::Double(value) => self.push_wide(StackValue::from_double(value)),
            JvmValue::Reference(value) => self.push(StackValue::from_reference(value))
        }
    }
}

impl Debug for InterpreterStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Local interpreter stack\n")?;
        for value in &mut self.stack.iter().rev() {
            write!(f, "\t{:?}\n", value)?;
        }
        Ok(())
    }
}
