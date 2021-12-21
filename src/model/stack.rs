use std::alloc::Layout;

use super::{
    types::JvmType,
    value::{JvmDouble, JvmFloat, JvmInt, JvmLong, JvmReference, JvmValue},
};

/// Points to the first empty slot (a slot is 4 bytes wide)
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct StackPointer(*mut u32);

impl StackPointer {
    pub fn with_size(slots: usize) -> Self {
        let layout = Layout::from_size_align(slots * 4, 4).unwrap();
        let stack = unsafe { std::alloc::alloc(layout) as *mut u32 };

        Self(stack)
    }

    pub fn start(&mut self) -> *mut u32 {
        self.0
    }

    pub fn reserve_slots(self, slots: usize) -> Self {
        Self(unsafe { self.0.offset(slots as isize) })
    }

    pub fn into_raw(self) -> *mut u32 {
        self.0
    }
}

pub struct StackFrame {
    frame_base: StackPointer,
    stack_end: StackPointer,
}

impl StackFrame {
    pub fn prepare(stack: StackPointer, parameters: usize, locals: usize) -> Self {
        let frame_base = stack;
        let stack_end = stack.reserve_slots(locals);

        for i in 0..parameters {
            unsafe {
                *frame_base.0.offset(i as isize) =
                    *frame_base.0.offset(-(parameters as isize) + i as isize);
            }
        }

        Self {
            frame_base,
            stack_end,
        }
    }

    pub fn get_stack_for_call(&mut self) -> StackPointer {
        self.stack_end
    }

    pub fn clear(self) -> StackPointer {
        self.frame_base
    }

    pub fn get_local(&self, index: usize) -> StackValue {
        unsafe { StackValue(*self.frame_base.0.offset(index as isize)) }
    }

    pub fn set_local(&self, index: usize, value: StackValue) {
        unsafe { *self.frame_base.0.offset(index as isize) = value.0 }
    }

    pub fn push(&mut self, value: StackValue) {
        unsafe {
            *self.stack_end.0 = value.0;
            self.stack_end.0 = self.stack_end.0.offset(1);
        }
    }

    pub fn push_wide(&mut self, values: (StackValue, StackValue)) {
        self.push(values.0);
        self.push(values.1);
    }

    pub fn push_value(&mut self, value: JvmValue, ty: JvmType) {
        match ty {
            JvmType::Void => {}
            JvmType::Integer => self.push(StackValue::from_int(value.int())),
            JvmType::Long => self.push_wide(StackValue::from_long(value.long())),
            JvmType::Float => self.push(StackValue::from_float(value.float())),
            JvmType::Double => self.push_wide(StackValue::from_double(value.double())),
            JvmType::Reference => self.push(StackValue::from_reference(value.reference())),
            _ => todo!(),
        }
    }

    pub fn pop(&mut self) -> StackValue {
        unsafe {
            self.stack_end.0 = self.stack_end.0.offset(-1);
            StackValue(*self.stack_end.0)
        }
    }

    pub fn pop_wide(&mut self) -> (StackValue, StackValue) {
        let top = self.pop();
        let second = self.pop();
        (second, top)
    }

    pub fn pop_type(&mut self, ty: JvmType) -> JvmValue {
        match ty {
            JvmType::Void => JvmValue::VOID,
            JvmType::Byte => todo!(),
            JvmType::Char => todo!(),
            JvmType::Integer => JvmValue {
                int: self.pop().as_int().into(),
            },
            JvmType::Long => JvmValue {
                long: self.pop_wide().as_long().into(),
            },
            JvmType::Float => JvmValue {
                float: self.pop().as_float().into(),
            },
            JvmType::Double => JvmValue {
                double: self.pop_wide().as_double().into(),
            },
            JvmType::Reference => JvmValue {
                reference: self.pop().as_reference().to_heap_index(),
            },
            JvmType::Short => todo!(),
            JvmType::Boolean => todo!(),
        }
    }

    pub fn peek(&self, offset: usize) -> StackValue {
        unsafe {
            // +1 because the stack pointer points to the first free slot and peek(0) should return the top value of the stack
            StackValue(*self.stack_end.0.offset(-(offset as isize + 1)))
        }
    }
}

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
        self.0.into()
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
        unsafe { Self(value.0.into_raw() as u32) }
    }

    pub fn to_raw(self) -> i32 {
        unsafe { std::mem::transmute::<u32, i32>(self.0) }
    }

    pub fn from_raw(value: i32) -> Self {
        unsafe { Self(std::mem::transmute::<i32, u32>(value)) }
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
