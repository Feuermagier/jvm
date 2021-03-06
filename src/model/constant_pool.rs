use std::fmt::Display;

use super::{
    class::VirtualMethodIndex, class_library::ClassIndex, field::FieldInfo, method::MethodIndex,
};

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

    pub fn get_class(
        &self,
        index: ConstantPoolIndex,
    ) -> Result<ConstantPoolIndex, ConstantPoolError> {
        let value = self.get(index)?;
        match value {
            ConstantPoolEntry::Class { name } => Ok(*name),
            _ => Err(ConstantPoolError::NotAClassReference(index, value.clone())),
        }
    }

    pub fn get_name_and_type(
        &self,
        index: ConstantPoolIndex,
    ) -> Result<(ConstantPoolIndex, ConstantPoolIndex), ConstantPoolError> {
        let value = self.get(index)?;
        match value {
            ConstantPoolEntry::NameAndType { name, ty } => Ok((*name, *ty)),
            _ => Err(ConstantPoolError::NotNameAndType(index, value.clone())),
        }
    }

    pub fn get_method(
        &self,
        index: ConstantPoolIndex,
    ) -> Result<MethodReference, ConstantPoolError> {
        let value = self.get(index)?;
        match value {
            ConstantPoolEntry::MethodReference(reference) => Ok(reference.clone()),
            _ => Err(ConstantPoolError::MethodNotResolvable(index)),
        }
    }

    pub fn update_resolved_field(
        &self,
        index: ConstantPoolIndex,
        info: FieldInfo,
        class: ClassIndex,
    ) {
        let entry = &self.entries[(index.0 - 1) as usize];
        let field = ConstantPoolEntry::FieldReference(FieldReference::Resolved { info, class });

        //Safety: This is a cache; nothing is changed here really and the cached value is constant
        unsafe {
            let entry = (entry as *const ConstantPoolEntry) as *mut ConstantPoolEntry;
            *entry = field;
        }
    }

    pub fn update_resolved_static_method(
        &self,
        index: ConstantPoolIndex,
        method: MethodIndex,
        parameter_count: usize,
    ) {
        let entry = &self.entries[(index.0 - 1) as usize];
        let field = ConstantPoolEntry::MethodReference(MethodReference::ResolvedStatic {
            index: method,
            parameter_count,
        });

        //Safety: This is a cache; nothing is changed here really and the cached value is constant
        unsafe {
            let entry = (entry as *const ConstantPoolEntry) as *mut ConstantPoolEntry;
            *entry = field;
        }
    }

    pub fn update_resolved_virtual_method(
        &self,
        index: ConstantPoolIndex,
        method_index: MethodIndex,
        virtual_index: VirtualMethodIndex,
        parameter_count: usize,
    ) {
        let entry = &self.entries[(index.0 - 1) as usize];
        let field = ConstantPoolEntry::MethodReference(MethodReference::ResolvedVirtual {
            method_index,
            virtual_index,
            parameter_count,
        });

        //Safety: This is a cache; nothing is changed here really and the cached value is constant
        unsafe {
            let entry = (entry as *const ConstantPoolEntry) as *mut ConstantPoolEntry;
            *entry = field;
        }
    }

    pub fn resolve_type(&self, index: ConstantPoolIndex) -> Result<&str, ConstantPoolError> {
        match self.get(index)? {
            ConstantPoolEntry::Class { name } => self.get_utf8(*name),
            _ => Err(ConstantPoolError::TypeNotResolvable(index)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ConstantPoolIndex(u16);

impl ConstantPoolIndex {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

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
    FieldReference(FieldReference),
    MethodReference(MethodReference),
    InterfaceMethodReference {
        class: ConstantPoolIndex,
        name_and_type: ConstantPoolIndex,
    },
    NameAndType {
        name: ConstantPoolIndex,
        ty: ConstantPoolIndex,
    },
    Empty, // To reserve the slot after longs and doubles
}

impl Display for ConstantPoolEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum FieldReference {
    Unresolved {
        class: ConstantPoolIndex,
        name_and_type: ConstantPoolIndex,
    },
    Resolved {
        info: FieldInfo,
        class: ClassIndex,
    },
}

#[derive(Debug, Clone)]
pub enum MethodReference {
    Unresolved {
        class: ConstantPoolIndex,
        name_and_type: ConstantPoolIndex,
    },
    ResolvedStatic {
        index: MethodIndex,
        parameter_count: usize,
    },
    ResolvedVirtual {
        method_index: MethodIndex,
        virtual_index: VirtualMethodIndex,
        parameter_count: usize,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum ConstantPoolError {
    #[error("there is no constant pool entry at {0}")]
    MissingEntry(ConstantPoolIndex),

    #[error("the value at index {0} is not loadable (according to JVM ??4.4 table 4.4-C")]
    NotLoadable(ConstantPoolIndex),

    #[error("the value at index {0} is not resolvable to a field")]
    FieldNotResolvable(ConstantPoolIndex),

    #[error("the value at index {0} is not resolvable to a class or interface type")]
    TypeNotResolvable(ConstantPoolIndex),

    #[error("the value at index {0} is not a valid type descriptor")]
    InvalidType(ConstantPoolIndex),

    #[error("the value at index {0} is not resolvable to a method reference")]
    MethodNotResolvable(ConstantPoolIndex),

    #[error("The constant pool entry at #{0} is expected to be of type UTF8, but is actually {1}")]
    NotAnUtf8String(ConstantPoolIndex, ConstantPoolEntry),

    #[error("The constant pool entry at {0} is expected to be of type class, but is actually {1}")]
    NotAClassReference(ConstantPoolIndex, ConstantPoolEntry),

    #[error(
        "The constant pool entry at {0} is expected to be of type NameAndType, but is actually {1}"
    )]
    NotNameAndType(ConstantPoolIndex, ConstantPoolEntry),
}
