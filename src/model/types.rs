use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq)]
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

impl Display for JvmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TypeError {
    #[error("Excpected type {0}, but got type {1}")]
    WrongType(JvmType, JvmType),
}
