use std::{fmt::Display, iter::Peekable};

use unicode_segmentation::Graphemes;

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
            _ => false,
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
            _ => false,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            JvmType::Void => 0,
            JvmType::Byte => 1,
            JvmType::Char => 1,
            JvmType::Integer => 4,
            JvmType::Long => 8,
            JvmType::Float => 4,
            JvmType::Double => 8,
            JvmType::Reference(_) => 2, // TODO
            JvmType::Short => 2,
            JvmType::Boolean => 1,
        }
    }

    pub fn parse(graphemes: &mut Peekable<Graphemes>) -> Option<JvmType> {
        let tag = graphemes.next();
        if tag.is_none() {
            return None;
        }

        match tag.unwrap() {
            "B" => Some(JvmType::Byte),
            "C" => Some(JvmType::Char),
            "D" => Some(JvmType::Double),
            "F" => Some(JvmType::Float),
            "I" => Some(JvmType::Integer),
            "J" => Some(JvmType::Long),
            "S" => Some(JvmType::Long),
            "Z" => Some(JvmType::Boolean),
            "V" => Some(JvmType::Void),
            "L" => {
                let class = graphemes.take_while(|c| *c != ";").collect::<String>();
                Some(JvmType::Reference(TypeReference::Unresolved(class)))
            }
            "[" => unimplemented!("Arrays are not implemented"),
            _ => None,
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
            (Self::Resolved(a), Self::Unresolved(b)) => classes.resolve(*a).name().unwrap() == b,
            (Self::Unresolved(a), Self::Resolved(b)) => classes.resolve(*b).name().unwrap() == a,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TypeError {
    #[error("Excpected type {0}, but got type {1}")]
    WrongType(String, String),
}
