use std::fmt::{self, Display};

use super::heap::HeapIndex;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(transparent)]
pub struct JvmInt(pub i32);

pub const JVM_GREATER: JvmInt = JvmInt(1);
pub const JVM_EQUAL: JvmInt = JvmInt(0);
pub const JVM_LESS: JvmInt = JvmInt(-1);

impl From<i32> for JvmInt {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<i8> for JvmInt {
    fn from(value: i8) -> Self {
        Self(value as i32)
    }
}

impl From<i16> for JvmInt {
    fn from(value: i16) -> Self {
        Self(value as i32)
    }
}

impl From<JvmInt> for i32 {
    fn from(value: JvmInt) -> Self {
        value.0
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(transparent)]
pub struct JvmFloat(pub f32);

impl From<f32> for JvmFloat {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<JvmFloat> for f32 {
    fn from(value: JvmFloat) -> Self {
        value.0
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(transparent)]
pub struct JvmLong(pub i64);

impl From<i64> for JvmLong {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<JvmLong> for i64 {
    fn from(value: JvmLong) -> Self {
        value.0
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(transparent)]
pub struct JvmDouble(pub f64);

impl From<f64> for JvmDouble {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<JvmDouble> for f64 {
    fn from(value: JvmDouble) -> Self {
        value.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct JvmReference(pub HeapIndex);

impl JvmReference {
    pub fn to_heap_index(self) -> HeapIndex {
        self.0
    }

    pub fn from_heap_index(index: HeapIndex) -> Self {
        Self(index)
    }
}

impl From<u32> for JvmReference {
    fn from(value: u32) -> Self {
        Self(HeapIndex::from_u32(value))
    }
}

impl From<JvmReference> for u32 {
    fn from(value: JvmReference) -> Self {
        value.0.as_u32()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union JvmValue {
    pub void: i64,
    pub int: i32,
    pub long: i64,
    pub float: f32,
    pub double: f64,
    pub reference: HeapIndex,
}

impl JvmValue {
    pub const VOID: JvmValue = JvmValue { void: 0 };

    pub fn int(self) -> JvmInt {
        unsafe { JvmInt(self.int) }
    }

    pub fn long(self) -> JvmLong {
        unsafe { JvmLong(self.long) }
    }

    pub fn float(self) -> JvmFloat {
        unsafe { JvmFloat(self.float) }
    }

    pub fn double(self) -> JvmDouble {
        unsafe { JvmDouble(self.double) }
    }

    pub fn reference(self) -> JvmReference {
        unsafe { JvmReference(self.reference) }
    }

    pub unsafe fn from_native(value: i64) -> Self {
        Self { void: value }
    }

    pub unsafe fn to_native(self) -> i64 {
        self.void
    }
}

impl Default for JvmValue {
    fn default() -> Self {
        JvmValue::VOID
    }
}

impl fmt::Debug for JvmValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { write!(f, "{:#b}", self.void) }
    }
}

impl Display for JvmValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "{:#b}", self.void) }
    }
}
