use std::{cell::RefCell, collections::HashMap};

use crate::{
    interpreter::{self, ExecutionError},
    model::{
        constant_pool::{ConstantPoolEntry, ConstantPoolError},
        value::JvmValue,
    },
};

use super::{
    constant_pool::{ConstantPool, ConstantPoolIndex},
    field::FieldDescriptor,
    heap::{Heap, HeapIndex},
    method::{Method, Parameters},
    types::JvmType,
    visibility::Visibility,
};

pub struct LoadedClasses {
    classes: Vec<Class>,
    name_mappings: HashMap<String, usize>,
}

impl LoadedClasses {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            name_mappings: HashMap::new(),
        }
    }

    pub fn resolve_by_name(&self, name: &str) -> &Class {
        &self.classes[*self.name_mappings.get(name).unwrap()]
    }

    pub fn resolve(&self, index: ClassIndex) -> &Class {
        &self.classes[index.0]
    }

    pub fn load(
        &mut self,
        constant_pool: ConstantPool,
        visibility: Visibility,
        this_class: ConstantPoolIndex,
        super_class: ConstantPoolIndex,
        interfaces: Vec<ConstantPoolIndex>,
        static_field_descriptors: Vec<FieldDescriptor>,
        field_descriptors: Vec<FieldDescriptor>,
        static_methods: Vec<Method>,
        methods: Vec<Method>,
    ) -> Result<ClassIndex, ConstantPoolError> {
        let index = self.classes.len();

        let class = Class::new(
            ClassIndex(index),
            constant_pool,
            visibility,
            this_class,
            super_class,
            interfaces,
            static_field_descriptors,
            field_descriptors,
            static_methods,
            methods,
        );
        self.name_mappings.insert(class.name()?.to_string(), index);
        self.classes.push(class);
        Ok(ClassIndex(index))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ClassIndex(usize);

pub struct Class {
    index: ClassIndex,

    constant_pool: ConstantPool,
    visibility: Visibility,
    this_class: ConstantPoolIndex,
    super_class: ConstantPoolIndex,
    interfaces: Vec<ConstantPoolIndex>,

    static_field_descriptors: Vec<FieldDescriptor>,
    static_field_index_map: HashMap<String, usize>,
    static_fields: RefCell<Vec<JvmValue>>,

    field_descriptors: Vec<FieldDescriptor>,
    field_index_map: HashMap<String, usize>,

    static_methods: HashMap<String, Method>,
    methods: HashMap<String, Method>,
}

impl Class {
    fn new(
        index: ClassIndex,
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
        dbg!(super_class);
        let static_field_count = static_field_descriptors.len();
        let mut static_field_index_map = HashMap::with_capacity(static_field_count);
        let mut static_fields = Vec::with_capacity(static_field_count);
        for desc in &static_field_descriptors {
            let index = static_fields.len();
            static_field_index_map.insert(desc.name.clone(), index);
            if let Some(value) = &desc.constant_value {
                static_fields.push(value.clone());
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
            index,
            constant_pool,
            visibility,
            this_class,
            super_class,
            interfaces,
            static_field_descriptors,
            static_field_index_map,
            static_fields: RefCell::new(static_fields),
            field_descriptors,
            field_index_map,
            static_methods,
            methods,
        }
    }

    pub fn bootstrap(
        &self,
        classes: &LoadedClasses,
        heap: &mut Heap,
    ) -> Result<(), ExecutionError> {
        let return_value =
            self.call_static_method("<clinit>", Parameters::empty(), classes, heap)?;
        return_value.assert_type(JvmType::Void)?;
        Ok(())
    }

    pub fn resolve_field(&self, index: ConstantPoolIndex) -> Result<&'_ str, ConstantPoolError> {
        match self.constant_pool.get(index)? {
            //TODO use the class
            ConstantPoolEntry::FieldReference { name_and_type, .. } => {
                match self.constant_pool.get(*name_and_type)? {
                    ConstantPoolEntry::NameAndType { name, .. } => {
                        self.constant_pool.get_utf8(*name)
                    }
                    _ => Err(ConstantPoolError::FieldNotResolvable(index)),
                }
            }
            _ => Err(ConstantPoolError::FieldNotResolvable(index)),
        }
    }

    pub fn resolve_type(&self, index: ConstantPoolIndex) -> Result<&str, ConstantPoolError> {
        match self.constant_pool.get(index)? {
            ConstantPoolEntry::Class { name } => self.constant_pool.get_utf8(*name),
            _ => Err(ConstantPoolError::TypeNotResolvable(index)),
        }
    }

    pub fn resolve_method(
        &self,
        index: ConstantPoolIndex,
    ) -> Result<(&str, &str, &str), ConstantPoolError> {
        match self.constant_pool.get(index)? {
            ConstantPoolEntry::MethodReference {
                class,
                name_and_type,
            } => match self.constant_pool.get(*name_and_type)? {
                ConstantPoolEntry::NameAndType { name, ty } => Ok((
                    self.resolve_type(*class)?,
                    self.constant_pool.get_utf8(*name)?,
                    self.constant_pool.get_utf8(*ty)?,
                )),
                _ => Err(ConstantPoolError::MethodNotResolvable(index)),
            },
            _ => Err(ConstantPoolError::MethodNotResolvable(index)),
        }
    }

    pub fn get_static_field(&self, name: &str) -> Result<JvmValue, FieldError> {
        if let Some(local_index) = self.static_field_index_map.get(name) {
            Ok(self.static_fields.borrow()[*local_index].clone())
        } else {
            Err(FieldError::UnknownStaticField(name.to_string()))
        }
    }

    pub fn set_static_field(&self, name: &str, value: JvmValue) -> Result<(), FieldError> {
        if let Some(local_index) = self.static_field_index_map.get(name) {
            self.static_fields.borrow_mut()[*local_index] = value;
            Ok(())
        } else {
            Err(FieldError::UnknownStaticField(name.to_string()))
        }
    }

    fn get_static_method(&self, name: &str) -> Result<&'_ Method, MethodError> {
        self.static_methods
            .get(name)
            .ok_or_else(|| MethodError::UnknownStaticMethod(name.to_string()))
    }

    fn get_method(&self, name: &str) -> Result<&'_ Method, MethodError> {
        self.methods
            .get(name)
            .ok_or_else(|| MethodError::UnknownVirtualMethod(name.to_string()))
    }

    pub fn call_static_method(
        &self,
        name: &str,
        parameters: Parameters,
        classes: &LoadedClasses,
        heap: &mut Heap,
    ) -> Result<JvmValue, ExecutionError> {
        // Resolve the method in super classes
        interpreter::execute_method(
            self.get_static_method(name)?,
            parameters,
            self,
            None,
            classes,
            heap,
        )
    }

    pub fn call_method(
        &self,
        name: &str,
        parameters: Parameters,
        this: HeapIndex,
        classes: &LoadedClasses,
        heap: &mut Heap,
    ) -> Result<JvmValue, ExecutionError> {
        // Resolve the method in super classes
        interpreter::execute_method(
            self.get_method(name)?,
            parameters,
            self,
            Some(this),
            classes,
            heap,
        )
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

    pub fn instantiate(&self) -> Instance {
        Instance {
            class: self.index,
            fields: self
                .field_descriptors
                .iter()
                .map(|field| {
                    if let Some(value) = &field.constant_value {
                        value.clone()
                    } else {
                        JvmValue::Void
                    }
                })
                .collect(),
        }
    }

    pub fn name(&self) -> Result<&str, ConstantPoolError> {
        self.resolve_type(self.this_class)
    }
}

pub struct Instance {
    class: ClassIndex,
    fields: Vec<JvmValue>,
}

impl Instance {
    pub fn get_field(&self, name: &str, classes: &LoadedClasses) -> Result<JvmValue, FieldError> {
        if let Some(local_index) = classes.resolve(self.class).field_index_map.get(name) {
            Ok(self.fields[*local_index].clone())
        } else {
            Err(FieldError::UnknownStaticField(name.to_string()))
        }
    }

    pub fn set_field(
        &mut self,
        name: &str,
        classes: &LoadedClasses,
        value: JvmValue,
    ) -> Result<(), FieldError> {
        if let Some(local_index) = classes.resolve(self.class).field_index_map.get(name) {
            self.fields[*local_index] = value;
            Ok(())
        } else {
            Err(FieldError::UnknownStaticField(name.to_string()))
        }
    }

    pub fn get_static_field(
        &self,
        name: &str,
        classes: &LoadedClasses,
    ) -> Result<JvmValue, FieldError> {
        classes.resolve(self.class).get_static_field(name)
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