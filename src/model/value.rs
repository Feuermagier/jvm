use std::fmt::Display;

use super::{heap::HeapIndex, types::{JvmType, TypeError}};

#[derive(Clone, Debug)]
pub enum JvmValue {
    Void,
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Reference(HeapIndex)
}

impl JvmValue {
    pub fn get_type(self) -> JvmType {
        match self {
            JvmValue::Void => JvmType::Void,
            JvmValue::Int(_) => JvmType::Integer,
            JvmValue::Long(_) => JvmType::Long,
            JvmValue::Float(_) => JvmType::Float,
            JvmValue::Double(_) => JvmType::Double,
            JvmValue::Reference(_) => JvmType::Reference,
        }
    }

    pub fn assert_type(self, ty: JvmType) -> Result<(), TypeError> {
        let own_type = self.get_type();
        if own_type == ty {
            Ok(())
        } else {
            Err(TypeError::WrongType(ty, own_type))
        }
    }

    pub fn as_int(self) -> Result<i32, TypeError> {
        match self {
            JvmValue::Int(value) => Ok(value),
            _ => Err(TypeError::WrongType(JvmType::Integer, self.get_type()))
        }
    }

    pub fn as_long(self) -> Result<i64, TypeError> {
        match self {
            JvmValue::Long(value) => Ok(value),
            _ => Err(TypeError::WrongType(JvmType::Long, self.get_type()))
        }
    }

    pub fn as_float(self) -> Result<f32, TypeError> {
        match self {
            JvmValue::Float(value) => Ok(value),
            _ => Err(TypeError::WrongType(JvmType::Float, self.get_type()))
        }
    }

    pub fn as_double(self) -> Result<f64, TypeError> {
        match self {
            JvmValue::Double(value) => Ok(value),
            _ => Err(TypeError::WrongType(JvmType::Double, self.get_type()))
        }
    }

    pub fn as_reference(self) -> Result<HeapIndex, TypeError> {
        match self {
            JvmValue::Reference(reference) => Ok(reference),
            _ => Err(TypeError::WrongType(JvmType::Reference, self.get_type()))
        }
    }
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
