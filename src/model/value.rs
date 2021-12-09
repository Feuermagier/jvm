use std::fmt::Display;

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
pub struct JvmReference(pub u16);

impl JvmReference {
    pub fn to_heap_index(self) -> HeapIndex {
        HeapIndex::from_u16(self.0)
    }

    pub fn from_heap_index(index: HeapIndex) -> Self {
        Self(index.as_u16())
    }
}

impl From<u16> for JvmReference {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<JvmReference> for u16 {
    fn from(value: JvmReference) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum JvmValue {
    Void,
    Int(JvmInt),
    Long(JvmLong),
    Float(JvmFloat),
    Double(JvmDouble),
    Reference(JvmReference),
}

impl Default for JvmValue {
    fn default() -> Self {
        Self::Void
    }
}

impl Display for JvmValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
