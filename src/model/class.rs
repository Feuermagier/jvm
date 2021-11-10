use std::{cell::RefCell, collections::HashMap, fmt::Display};

use crate::{
    interpreter::{self, ExecutionError},
    model::value::JvmValue,
};

use super::{
    field::FieldDescriptor,
    method::{Method, Parameters},
    types::JvmType,
    visibility::Visibility,
};

pub struct Class {
    constant_pool: ConstantPool,
    visibility: Visibility,
    this_class: ConstantPoolIndex,
    super_class: ConstantPoolIndex,
    interfaces: Vec<ConstantPoolIndex>,

    static_field_descriptors: Vec<FieldDescriptor>,
    static_field_index_map: HashMap<String, usize>,
    static_fields: Vec<JvmValue>,

    field_descriptors: Vec<FieldDescriptor>,
    field_index_map: HashMap<String, usize>,

    static_methods: HashMap<String, Method>,
    methods: HashMap<String, Method>,
}

impl Class {
    pub fn new(
        constant_pool: ConstantPool,
        visibility: Visibility,
        this_class: ConstantPoolIndex,
        super_class: ConstantPoolIndex,
        interfaces: Vec<ConstantPoolIndex>,
        static_field_descriptors: Vec<FieldDescriptor>,
        field_descriptors: Vec<FieldDescriptor>,
        mut static_methods: Vec<Method>,
        mut methods: Vec<Method>,
    ) -> Self {
        let static_field_count = static_field_descriptors.len();
        let mut static_field_index_map = HashMap::with_capacity(static_field_count);
        let mut static_fields = Vec::with_capacity(static_field_count);
        for desc in &static_field_descriptors {
            let index = static_fields.len();
            static_field_index_map.insert(desc.name.clone(), index);
            if let Some(value) = desc.constant_value {
                static_fields.push(value);
            } else {
                static_fields.push(JvmValue::Void);
            }
        }

        let field_count = field_descriptors.len();
        let mut field_index_map = HashMap::with_capacity(field_count);
        for (i, desc) in field_descriptors.iter().enumerate() {
            field_index_map.insert(desc.name.clone(), i);
        }

        let static_methods = static_methods
            .drain(..)
            .map(|method| (method.name.clone(), method))
            .collect();
        let methods = methods
            .drain(..)
            .map(|method| (method.name.clone(), method))
            .collect();

        Self {
            constant_pool,
            visibility,
            this_class,
            super_class,
            interfaces,
            static_field_descriptors,
            static_field_index_map,
            static_fields,
            field_descriptors,
            field_index_map,
            static_methods,
            methods,
        }
    }

    pub fn bootstrap(&mut self) -> Result<(), ExecutionError> {
        let return_value = self.call_static_method("<clinit>", Parameters::empty())?;
        return_value.assert_type(JvmType::Void)?;
        Ok(())
    }

    pub fn resolve_field(&self, index: ConstantPoolIndex) -> Result<&'_ str, ConstantPoolError> {
        match self.constant_pool.get(index)? {
            //TODO use the class
            ConstantPoolEntry::FieldReference {name_and_type, .. } =>  {
                match self.constant_pool.get(*name_and_type)? {
                    ConstantPoolEntry::NameAndType { name, .. } => {
                        self.constant_pool.get_utf8(*name)
                    },
                    _ => Err(ConstantPoolError::FieldNotResolvable(index))
                }
            },
            _ => Err(ConstantPoolError::FieldNotResolvable(index))
        }
    }

    pub fn get_static_field(&self, name: &str) -> Result<JvmValue, FieldError> {
        if let Some(local_index) = self.static_field_index_map.get(name) {
            Ok(self.static_fields[*local_index])
        } else {
            Err(FieldError::UnknownStaticField(name.to_string()))
        }
    }

    pub fn set_static_field(&mut self, name: &str, value: JvmValue) -> Result<(), FieldError> {
        if let Some(local_index) = self.static_field_index_map.get(name) {
            self.static_fields[*local_index] = value;
            Ok(())
        } else {
            Err(FieldError::UnknownStaticField(name.to_string()))
        }
    }

    pub fn get_static_method(&self, name: &str) -> Result<&'_ Method, MethodError> {
        self.static_methods
            .get(name)
            .ok_or_else(|| MethodError::UnknownStaticMethod(name.to_string()))
    }

    pub fn get_method(&self, name: &str) -> Result<&'_ Method, MethodError> {
        self.methods
            .get(name)
            .ok_or_else(|| MethodError::UnknownVirtualMethod(name.to_string()))
    }

    pub fn call_static_method(
        &mut self,
        name: &str,
        parameters: Parameters,
    ) -> Result<JvmValue, ExecutionError> {
        interpreter::execute_method(name, parameters, self, None)
    }

    pub fn get_loadable(&self, index: ConstantPoolIndex) -> Result<JvmValue, ConstantPoolError> {
        let value = self.constant_pool.get(index)?;
        match value {
            ConstantPoolEntry::Integer(value) => Ok(JvmValue::Int(*value)),
            ConstantPoolEntry::Long(value) => Ok(JvmValue::Long(*value)),
            ConstantPoolEntry::Float(value) => Ok(JvmValue::Float(*value)),
            ConstantPoolEntry::Double(value) => Ok(JvmValue::Double(*value)),
            ConstantPoolEntry::String(_) => todo!(),
            ConstantPoolEntry::Class { .. } => todo!(),
            // + MethodHandle, MethodType, Dynamic
            _ => Err(ConstantPoolError::NotLoadable(index)),
        }
    }
}

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
}

impl Display for ConstantPoolEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Instance<'c> {
    class: &'c Class,
    fields: Vec<JvmValue>,
}

impl<'c> Instance<'c> {
    pub fn new(class: &'c Class, fields: &Vec<FieldDescriptor>) -> Self {
        Self {
            class,
            fields: fields
                .iter()
                .map(|field| {
                    if let Some(value) = field.constant_value {
                        value
                    } else {
                        JvmValue::Void
                    }
                })
                .collect(),
        }
    }

    pub fn get_field(&self, name: &str) -> Result<JvmValue, FieldError> {
        if let Some(local_index) = self.class.field_index_map.get(name) {
            Ok(self.fields[*local_index])
        } else {
            Err(FieldError::UnknownStaticField(name.to_string()))
        }
    }

    pub fn get_static_field(&self, name: &str) -> Result<JvmValue, FieldError> {
        self.class.get_static_field(name)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FieldError {
    #[error("Unknown instance field '#{0}'")]
    UnknownInstanceField(String),

    #[error("Unknown static field '#{0}'")]
    UnknownStaticField(String),
}

#[derive(thiserror::Error, Debug)]
pub enum MethodError {
    #[error("Unknown instance method '#{0}'")]
    UnknownVirtualMethod(String),

    #[error("Unknown static method '#{0}'")]
    UnknownStaticMethod(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ConstantPoolError {
    #[error("there is no constant pool entry at {0}")]
    MissingEntry(ConstantPoolIndex),

    #[error("the value at index {0} is not loadable (according to JVM §4.4 table 4.4-C")]
    NotLoadable(ConstantPoolIndex),

    #[error("the value at index {0} is not resolvable to a field")]
    FieldNotResolvable(ConstantPoolIndex),

    #[error(
        "The constant pool entry at #{0} is expected to be of type utf 8, but is actually {1}"
    )]
    NotAnUtf8String(ConstantPoolIndex, ConstantPoolEntry),
}

/*
bitflags::bitflags! {
    pub struct ClassAccessModifiers: u16 {
        const PUBLIC = 0x0001;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
        const MODULE = 0x8000;
    }
}
*/
