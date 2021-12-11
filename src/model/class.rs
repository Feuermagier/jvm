use std::{cell::RefCell, collections::HashMap};

use crate::{
    class_parser::ClassData,
    interpreter::{self, ExecutionError},
    model::constant_pool::{ConstantPoolEntry, ConstantPoolError},
};

use super::{
    class_library::{ClassIndex, ClassLibrary},
    constant_pool::{ConstantPool, ConstantPoolIndex, FieldReference},
    field::FieldDescriptor,
    fields::Fields,
    heap::{Heap, HeapIndex},
    method::{Method, MethodDescriptor, Parameters},
    types::JvmType,
    value::JvmValue,
};

pub struct Class {
    index: ClassIndex,
    super_class: Option<ClassIndex>,
    data: ClassData,
    constant_pool: ConstantPool,

    static_field_offsets: HashMap<String, (usize, JvmType)>,
    static_fields: RefCell<Fields>,

    fields_size: usize,
    field_offsets: HashMap<String, (usize, JvmType)>,

    static_methods: HashMap<String, Method>,
    methods: HashMap<String, Method>,
}

impl Class {
    pub fn new(
        mut data: ClassData,
        constant_pool: ConstantPool,
        index: ClassIndex,
        super_class_index: Option<ClassIndex>,
    ) -> Result<Self, ClassCreationError> {
        let (static_fields_size, static_field_offsets) = place_fields(&data.static_fields);
        let (fields_size, field_offsets) = place_fields(&data.fields);

        let static_fields = init_fields(
            &data.static_fields,
            &static_field_offsets,
            static_fields_size,
        );

        let static_methods = data
            .static_methods
            .iter()
            .map(|method| (method.name.clone(), setup_method(method)))
            .collect();
        let methods = data
            .methods
            .iter()
            .map(|method| (method.name.clone(), setup_method(method)))
            .collect();

        Ok(Self {
            index,
            data,
            super_class: super_class_index,
            constant_pool,
            static_field_offsets,
            static_fields: RefCell::new(static_fields),
            fields_size,
            field_offsets,
            static_methods,
            methods,
        })
    }

    pub fn update_class_index(&mut self, index: ClassIndex) {
        self.index = index;
    }

    pub fn bootstrap(&self, classes: &ClassLibrary, heap: &mut Heap) -> Result<(), ExecutionError> {
        if self.static_methods.contains_key("<clinit>") {
            let _return_value =
                self.call_static_method("<clinit>", Parameters::empty(), classes, heap)?;
        }
        Ok(())
    }

    pub fn resolve_field(
        &self,
        index: ConstantPoolIndex,
        static_field: bool,
        classes: &ClassLibrary,
        heap: &mut Heap,
    ) -> Result<(ClassIndex, FieldInfo), ConstantPoolError> {
        match self.constant_pool.get(index)? {
            //TODO use the class
            ConstantPoolEntry::FieldReference(reference) => match reference {
                FieldReference::Resolved { info, class } => Ok((*class, *info)),
                FieldReference::Unresolved {
                    name_and_type,
                    class,
                } => {
                    let (name, ty) = self.constant_pool.get_name_and_type(*name_and_type)?;
                    //let ty_str = self.constant_pool.get_utf8(ty)?;
                    let name = self.constant_pool.get_utf8(name)?;

                    let callee_class_name = self
                        .constant_pool
                        .get_utf8(self.constant_pool.get_class(*class)?)?;
                    let callee_class = classes.resolve_by_name(callee_class_name, heap);

                    let (offset, ty) = if static_field {
                        &callee_class.static_field_offsets
                    } else {
                        &callee_class.field_offsets
                    }
                    .get(name)
                    .ok_or(ConstantPoolError::FieldNotResolvable(index))?;
                    let info = FieldInfo {
                        offset: *offset,
                        ty: *ty,
                    };

                    self.constant_pool
                        .update_resolved_field(index, info, callee_class.index());

                    Ok((callee_class.index(), info))
                }
            },
            _ => Err(ConstantPoolError::FieldNotResolvable(index)),
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
                    self.constant_pool.resolve_type(*class)?,
                    self.constant_pool.get_utf8(*name)?,
                    self.constant_pool.get_utf8(*ty)?,
                )),
                _ => Err(ConstantPoolError::MethodNotResolvable(index)),
            },
            _ => Err(ConstantPoolError::MethodNotResolvable(index)),
        }
    }

    pub fn get_static_field(&self, info: &FieldInfo) -> JvmValue {
        self.static_fields.borrow().get_value(info.offset, info.ty)
    }

    pub fn get_static_field_by_name(&self, name: &str) -> JvmValue {
        let (offset, ty) = *self.static_field_offsets.get(name).unwrap();
        self.get_static_field(&FieldInfo { offset, ty })
    }

    pub fn set_static_field(&self, info: FieldInfo, value: JvmValue) {
        self.static_fields
            .borrow_mut()
            .set_value(info.offset, value);
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
        &self,
        name: &str,
        parameters: Parameters,
        classes: &ClassLibrary,
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
        classes: &ClassLibrary,
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
            ConstantPoolEntry::Integer(value) => Ok(JvmValue::Int((*value).into())),
            ConstantPoolEntry::Long(value) => Ok(JvmValue::Long((*value).into())),
            ConstantPoolEntry::Float(value) => Ok(JvmValue::Float((*value).into())),
            ConstantPoolEntry::Double(value) => Ok(JvmValue::Double((*value).into())),
            ConstantPoolEntry::String(_) => todo!(),
            ConstantPoolEntry::Class { .. } => todo!(),
            // + MethodHandle, MethodType, Dynamic
            _ => Err(ConstantPoolError::NotLoadable(index)),
        }
    }

    pub fn instantiate(&self) -> Instance {
        Instance {
            class: self.index,
            fields: init_fields(&self.data.fields, &self.field_offsets, self.fields_size),
        }
    }

    pub fn name(&self) -> Result<&str, ConstantPoolError> {
        self.constant_pool.resolve_type(self.data.this_class)
    }

    pub fn index(&self) -> ClassIndex {
        self.index
    }

    pub fn resolve_type(&self, index: ConstantPoolIndex) -> Result<&str, ConstantPoolError> {
        self.constant_pool.resolve_type(index)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FieldInfo {
    pub offset: usize,
    pub ty: JvmType,
}

pub struct Instance {
    class: ClassIndex,
    fields: Fields,
}

impl Instance {
    pub fn get_field(&self, info: FieldInfo) -> JvmValue {
        self.fields.get_value(info.offset, info.ty)
    }

    pub fn set_field(&mut self, info: FieldInfo, value: JvmValue) {
        self.fields.set_value(info.offset, value);
    }

    pub fn class(&self) -> ClassIndex {
        self.class
    }
}

fn place_fields(descriptors: &Vec<FieldDescriptor>) -> (usize, HashMap<String, (usize, JvmType)>) {
    let count = descriptors.len();
    let mut offset_map = HashMap::with_capacity(count);
    let mut offset = 0;
    for desc in descriptors {
        let size = desc.ty.size();
        offset_map.insert(desc.name.clone(), (offset, desc.ty.clone()));
        offset += size;
    }

    (offset, offset_map)
}

fn init_fields(
    descriptors: &Vec<FieldDescriptor>,
    offsets: &HashMap<String, (usize, JvmType)>,
    length: usize,
) -> Fields {
    let mut fields = Fields::new(length);
    for desc in descriptors {
        if let Some(value) = desc.constant_value {
            let (offset, ty) = &offsets[&desc.name];
            fields.set_value(*offset, value);
        }
    }
    fields
}

fn setup_method(descriptor: &MethodDescriptor) -> Method {
    if descriptor.code.is_some() {
        Method::new_bytecode_method(descriptor)
    } else {
        log::warn!(
            "Native method '{}', binding a noop implementation",
            &descriptor.name
        );
        Method::new_native_method(descriptor, Box::new(|_| JvmValue::Void))
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
pub enum ClassCreationError {
    #[error("Failed to resolve the super class")]
    SuperclassResolutionFailed(#[from] ConstantPoolError),
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
