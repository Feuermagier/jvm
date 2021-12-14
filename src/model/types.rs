use std::{fmt::Display, iter::Peekable};

use unicode_segmentation::Graphemes;

use super::class_library::{ClassIndex, ClassLibrary};

#[derive(Clone, Copy, Debug)]
pub enum JvmType {
    Void,
    Byte,
    Char,
    Integer,
    Long,
    Float,
    Double,
    Reference,
    Short,
    Boolean,
    // + arrays
}

impl JvmType {
    pub fn matches(self, other: &Self) -> bool {
        match (self, other) {
            (JvmType::Void, JvmType::Void) => true,
            (JvmType::Byte, JvmType::Byte) => true,
            (JvmType::Char, JvmType::Char) => true,
            (JvmType::Integer, JvmType::Integer) => true,
            (JvmType::Long, JvmType::Long) => true,
            (JvmType::Float, JvmType::Float) => true,
            (JvmType::Double, JvmType::Double) => true,
            (JvmType::Reference, JvmType::Reference) => true,
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
            JvmType::Reference => 2, // TODO
            JvmType::Short => 2,
            JvmType::Boolean => 1,
        }
    }

    pub fn alignment(&self) -> usize {
        self.size()
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
                // We have to read the class even though we don't use it currently so that
                // the iterator gets advanced
                let _ = graphemes.take_while(|c| *c != ";").collect::<String>();
                //Some(JvmType::Reference(TypeReference::Unresolved(class)))
                Some(JvmType::Reference)
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
    pub fn matches(&self, other: &Self, classes: &ClassLibrary) -> bool {
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
