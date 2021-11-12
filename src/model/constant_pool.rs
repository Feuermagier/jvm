use std::fmt::Display;

#[derive(Debug)]
pub struct ConstantPool {
    entries: Vec<ConstantPoolEntry>,
}

impl ConstantPool {
    pub fn new(entries: Vec<ConstantPoolEntry>) -> Self {
        Self { entries }
    }

    pub fn get(
        &self,
        index: ConstantPoolIndex,
    ) -> Result<&'_ ConstantPoolEntry, ConstantPoolError> {
        self.entries
            .get((index.0 - 1) as usize)
            .ok_or(ConstantPoolError::MissingEntry(index))
    }

    pub fn get_utf8(&self, index: ConstantPoolIndex) -> Result<&'_ str, ConstantPoolError> {
        let value = self.get(index)?;
        match value {
            ConstantPoolEntry::Utf8(string) => Ok(string),
            _ => Err(ConstantPoolError::NotAnUtf8String(index, value.clone())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ConstantPoolIndex(u16);

impl From<u16> for ConstantPoolIndex {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

impl Display for ConstantPoolIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum ConstantPoolEntry {
    Utf8(String),
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Class {
        name: ConstantPoolIndex,
    },
    FieldReference {
        class: ConstantPoolIndex,
        name_and_type: ConstantPoolIndex,
    },
    MethodReference {
        class: ConstantPoolIndex,
        name_and_type: ConstantPoolIndex,
    },
    InterfaceMethodReference {
        class: ConstantPoolIndex,
        name_and_type: ConstantPoolIndex,
    },
    NameAndType {
        name: ConstantPoolIndex,
        ty: ConstantPoolIndex,
    },
    Empty // To reserve the slot after longs and doubles
}

impl Display for ConstantPoolEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(thiserror::Error, Debug)]
pub enum ConstantPoolError {
    #[error("there is no constant pool entry at {0}")]
    MissingEntry(ConstantPoolIndex),

    #[error("the value at index {0} is not loadable (according to JVM §4.4 table 4.4-C")]
    NotLoadable(ConstantPoolIndex),

    #[error("the value at index {0} is not resolvable to a field")]
    FieldNotResolvable(ConstantPoolIndex),

    #[error("the value at index {0} is not resolvable to a type")]
    TypeNotResolvable(ConstantPoolIndex),

    #[error("the value at index {0} is not resolvable to a method reference")]
    MethodNotResolvable(ConstantPoolIndex),

    #[error(
        "The constant pool entry at #{0} is expected to be of type utf 8, but is actually {1}"
    )]
    NotAnUtf8String(ConstantPoolIndex, ConstantPoolEntry),
}