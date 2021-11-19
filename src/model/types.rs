use std::fmt::Display;

use super::class::{ClassIndex, LoadedClasses};

#[derive(Clone, Debug)]
pub enum JvmType {
    Void,
    Byte,
    Char,
    Integer,
    Long,
    Float,
    Double,
    Reference(TypeReference),
    Short,
    Boolean,
    // + arrays
}

impl JvmType {
    pub fn matches(self, other: &Self, classes: &LoadedClasses) -> bool {
        match (self, other) {
            (JvmType::Void, JvmType::Void) => true,
            (JvmType::Byte, JvmType::Byte) => true,
            (JvmType::Char, JvmType::Char) => true,
            (JvmType::Integer, JvmType::Integer) => true,
            (JvmType::Long, JvmType::Long) => true,
            (JvmType::Float, JvmType::Float) => true,
            (JvmType::Double, JvmType::Double) => true,
            (JvmType::Reference(a), JvmType::Reference(b)) => a.matches(b, classes),
            (JvmType::Short, JvmType::Short) => true,
            (JvmType::Boolean, JvmType::Boolean) => true,
            _ => false
        }
    }

    pub fn matches_ignoring_references(self, other: &Self) -> bool {
        match (self, other) {
            (JvmType::Void, JvmType::Void) => true,
            (JvmType::Byte, JvmType::Byte) => true,
            (JvmType::Char, JvmType::Char) => true,
            (JvmType::Integer, JvmType::Integer) => true,
            (JvmType::Long, JvmType::Long) => true,
            (JvmType::Float, JvmType::Float) => true,
            (JvmType::Double, JvmType::Double) => true,
            (JvmType::Reference(_), JvmType::Reference(_)) => true,
            (JvmType::Short, JvmType::Short) => true,
            (JvmType::Boolean, JvmType::Boolean) => true,
            _ => false
        }
    }
}

impl Display for JvmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub enum TypeReference {
    Resolved(ClassIndex),
    Unresolved(String),
}

impl TypeReference {
    pub fn matches(&self, other: &Self, classes: &LoadedClasses) -> bool {
        match (self, other) {
            (Self::Resolved(a), Self::Resolved(b)) => a == b,
            (Self::Unresolved(a), Self::Unresolved(b)) => a == b,
            (Self::Resolved(a), Self::Unresolved(b)) => classes.resolve_by_name(b).index() == *a,
            (Self::Unresolved(a), Self::Resolved(b)) => classes.resolve_by_name(a).index() == *b,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TypeError {
    #[error("Excpected type {0}, but got type {1}")]
    WrongType(String, String),
}
