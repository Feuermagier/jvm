use std::fmt::Display;

use super::{class::LoadedClasses, heap::{Heap, HeapIndex}, types::{JvmType, TypeError, TypeReference}};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum JvmValue {
    Void,
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Reference(HeapIndex)
}

impl JvmValue {
    pub fn get_type(self, heap: &Heap) -> JvmType {
        match self {
            JvmValue::Void => JvmType::Void,
            JvmValue::Int(_) => JvmType::Integer,
            JvmValue::Long(_) => JvmType::Long,
            JvmValue::Float(_) => JvmType::Float,
            JvmValue::Double(_) => JvmType::Double,
            JvmValue::Reference(index) => {
                JvmType::Reference(TypeReference::Resolved(heap.resolve(index).class()))
            },
        }
    }

    pub fn assert_type(self, ty: JvmType, heap: &Heap, classes: &LoadedClasses) -> Result<(), TypeError> {
        let own_type = self.get_type(heap);
        if own_type.matches(&ty, classes) {
            Ok(())
        } else {
            Err(TypeError::WrongType(ty.to_string(), self.get_type_simple_string()))
        }
    }

    pub fn as_int(self) -> Result<i32, TypeError> {
        match self {
            JvmValue::Int(value) => Ok(value),
            _ => Err(TypeError::WrongType("Integer".to_string(), self.get_type_simple_string()))
        }
    }

    pub fn as_long(self) -> Result<i64, TypeError> {
        match self {
            JvmValue::Long(value) => Ok(value),
            _ => Err(TypeError::WrongType("Long".to_string(), self.get_type_simple_string()))
        }
    }

    pub fn as_float(self) -> Result<f32, TypeError> {
        match self {
            JvmValue::Float(value) => Ok(value),
            _ => Err(TypeError::WrongType("Float".to_string(), self.get_type_simple_string()))
        }
    }

    pub fn as_double(self) -> Result<f64, TypeError> {
        match self {
            JvmValue::Double(value) => Ok(value),
            _ => Err(TypeError::WrongType("Double".to_string(), self.get_type_simple_string()))
        }
    }

    pub fn as_reference(self) -> Result<HeapIndex, TypeError> {
        match self {
            JvmValue::Reference(reference) => Ok(reference),
            _ => Err(TypeError::WrongType("Reference".to_string(), self.get_type_simple_string()))
        }
    }

    fn get_type_simple_string(self) -> String {
        match self {
            JvmValue::Void => "Void".to_string(),
            JvmValue::Int(_) => "Integer".to_string(),
            JvmValue::Long(_) => "Long".to_string(),
            JvmValue::Float(_) => "Float".to_string(),
            JvmValue::Double(_) => "Double".to_string(),
            JvmValue::Reference(_) => "Reference".to_string(),
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
