use std::{cell::RefCell, collections::HashMap};

use crate::{
    class_parser::ClassData,
    interpreter::{self, ExecutionError},
    model::constant_pool::{ConstantPoolEntry, ConstantPoolError},
};

use super::{
    class_library::{ClassIndex, ClassLibrary},
    constant_pool::{ConstantPool, ConstantPoolIndex, FieldReference, MethodReference},
    field::{self, FieldDescriptor, FieldInfo, FieldLayout, Fields},
    heap::Heap,
    method::{Method, MethodIndex, MethodTable, Parameters},
    types::JvmType,
    value::JvmValue,
};

pub struct Class {
    index: ClassIndex,
    super_class: Option<ClassIndex>,
    data: ClassData,
    constant_pool: ConstantPool,

    static_field_layout: FieldLayout,
    static_fields: RefCell<Fields>,

    field_layout: FieldLayout,

    static_methods: HashMap<String, (MethodIndex, usize)>,
    virtual_methods: HashMap<String, (MethodIndex, VirtualMethodIndex, usize)>, // The MethodIndex is used for static dispatch (i.e. invokespecial)
    dispatch_table: Vec<MethodIndex>,
}

impl Class {
    pub fn new(
        data: ClassData,
        constant_pool: ConstantPool,
        index: ClassIndex,
        super_class: Option<&Class>,
        methods: &MethodTable,
    ) -> Result<Self, ClassCreationError> {
        let static_field_layout = field::layout_fields(&FieldLayout::empty(), &data.static_fields);
        let static_fields = Fields::init_from_layout(&static_field_layout);

        let field_layout = if let Some(super_class) = super_class {
            let super_field_layout = &super_class.field_layout;
            field::layout_fields(super_field_layout, &data.fields)
        } else {
            field::layout_fields(&FieldLayout::empty(), &data.fields)
        };

        let mut static_methods = if let Some(super_class) = super_class {
            super_class.static_methods.clone()
        } else {
            HashMap::new()
        };
        for desc in &data.static_methods {
            if let Some(code) = &desc.code {
                let name = desc.name.clone();
                let code = code.clone();
                let max_stack = desc.max_stack;
                let max_locals = desc.max_locals;
                let method_index = methods.add_method(Box::new(
                    move |heap, classes, methods, this, parameters| {
                        let method = Method {
                            name: &name,
                            code: &code,
                            max_stack,
                            max_locals,
                        };
                        interpreter::execute_method(
                            method, parameters, index, this, classes, heap, methods,
                        )
                        .unwrap()
                    },
                ));
                // Quite sure parameters.len() isn't correct: Parameter count should be in words (i.e. 4 bytes), but parameter_count will be e.g. 1 for a double
                static_methods.insert(desc.name.to_string(), (method_index, desc.parameters.len()));
            }
        }

        let mut virtual_methods = if let Some(super_class) = super_class {
            super_class.virtual_methods.clone()
        } else {
            HashMap::new()
        };
        let mut dispatch_table = if let Some(super_class) = super_class {
            super_class.dispatch_table.clone()
        } else {
            Vec::new()
        };

        for desc in &data.methods {
            if let Some(code) = &desc.code {
                let name = desc.name.clone();
                let code = code.clone();
                let max_stack = desc.max_stack;
                let max_locals = desc.max_locals;
                let method_index = methods.add_method(Box::new(
                    move |heap, classes, methods, this, parameters| {
                        let method = Method {
                            name: &name,
                            code: &code,
                            max_stack,
                            max_locals,
                        };
                        interpreter::execute_method(
                            method, parameters, index, this, classes, heap, methods,
                        )
                        .unwrap()
                    },
                ));

                if let Some((old_method_index, virtual_index, _)) =
                    virtual_methods.get_mut(&desc.name)
                {
                    dispatch_table[virtual_index.0] = method_index;
                    *old_method_index = method_index;
                } else {
                    let virtual_index = dispatch_table.len();
                    dispatch_table.push(method_index);
                    // Quite sure parameters.len() isn't correct: Parameter count should be in words (i.e. 4 bytes), but parameter_count will be e.g. 1 for a double
                    virtual_methods.insert(
                        desc.name.to_string(),
                        (
                            method_index,
                            VirtualMethodIndex(virtual_index),
                            desc.parameters.len(),
                        ),
                    );
                }
            }
        }

        Ok(Self {
            index,
            data,
            super_class: super_class.map(|class| class.index()),
            constant_pool,
            static_field_layout,
            static_fields: RefCell::new(static_fields),
            field_layout,
            static_methods,
            virtual_methods,
            dispatch_table,
        })
    }

    pub fn update_class_index(&mut self, index: ClassIndex) {
        self.index = index;
    }

    pub fn bootstrap(
        &self,
        methods: &MethodTable,
        classes: &ClassLibrary,
        heap: &mut Heap,
    ) -> Result<(), ExecutionError> {
        if let Some((clinit, _)) = self.static_methods.get("<clinit>") {
            let _return_value =
                methods.resolve(*clinit)(heap, classes, methods, None, Parameters::empty());
        }
        Ok(())
    }

    //todo: This will definitely not work for shadowed fields
    pub fn resolve_instance_field(
        &self,
        index: ConstantPoolIndex,
        classes: &ClassLibrary,
        heap: &mut Heap,
        methods: &MethodTable,
    ) -> Result<FieldInfo, FieldError> {
        match self.constant_pool.get(index)? {
            ConstantPoolEntry::FieldReference(reference) => match reference {
                FieldReference::Resolved { info, .. } => Ok(*info),
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
                    let callee_class = classes.resolve_by_name(callee_class_name, methods, heap);

                    let info = callee_class.field_layout.resolve(name)?;

                    self.constant_pool
                        .update_resolved_field(index, info, callee_class.index());

                    Ok(info)
                }
            },
            _ => Err(FieldError::ConstantPool(
                ConstantPoolError::FieldNotResolvable(index),
            )),
        }
    }

    pub fn resolve_static_field(
        &self,
        index: ConstantPoolIndex,
        classes: &ClassLibrary,
        heap: &mut Heap,
        methods: &MethodTable,
    ) -> Result<(ClassIndex, FieldInfo), FieldError> {
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

                    let (owning_class, info) = classes
                        .resolve_by_name(callee_class_name, methods, heap)
                        .resolve_own_static_field(name, classes)?;

                    self.constant_pool
                        .update_resolved_field(index, info, owning_class);

                    Ok((owning_class, info))
                }
            },
            _ => Err(FieldError::ConstantPool(
                ConstantPoolError::FieldNotResolvable(index),
            )),
        }
    }

    fn resolve_own_static_field(
        &self,
        name: &str,
        classes: &ClassLibrary,
    ) -> Result<(ClassIndex, FieldInfo), FieldError> {
        if let Ok(info) = self.static_field_layout.resolve(name) {
            Ok((self.index(), info))
        } else if let Some(super_class) = self.super_class {
            classes
                .resolve(super_class)
                .resolve_own_static_field(name, classes)
        } else {
            Err(FieldError::StaticFieldNotFound(name.to_string()))
        }
    }

    /*
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
    */

    pub fn resolve_static_method(
        &self,
        index: ConstantPoolIndex,
        classes: &ClassLibrary,
        heap: &mut Heap,
        methods: &MethodTable,
    ) -> Result<(MethodIndex, usize), MethodError> {
        match self.constant_pool.get_method(index)? {
            MethodReference::ResolvedStatic {
                index,
                argument_count,
            } => Ok((index, argument_count)),
            MethodReference::Unresolved {
                class,
                name_and_type,
            } => {
                let (name, ty) = self.constant_pool.get_name_and_type(name_and_type)?;
                let callee_class = self.constant_pool.resolve_type(class)?;
                let name = self.constant_pool.get_utf8(name)?;

                let method = classes
                    .resolve_by_name(callee_class, methods, heap)
                    .static_methods
                    .get(name)
                    .ok_or_else(|| MethodError::UnknownStatic(name.to_string()))?;

                self.constant_pool
                    .update_resolved_static_method(index, method.0, method.1);

                Ok(*method)
            }
            _ => Err(MethodError::NotStatic(index)),
        }
    }

    pub fn resolve_own_static_method_by_name(&self, name: &str) -> (MethodIndex, usize) {
        *self.static_methods.get(name).unwrap()
    }

    pub fn resolve_virtual_method_statically(
        &self,
        index: ConstantPoolIndex,
        classes: &ClassLibrary,
        heap: &mut Heap,
        methods: &MethodTable,
    ) -> Result<(MethodIndex, usize), MethodError> {
        match self.constant_pool.get_method(index)? {
            MethodReference::ResolvedStatic {
                index,
                argument_count,
            } => Ok((index, argument_count)),
            MethodReference::Unresolved {
                class,
                name_and_type,
            } => {
                let (name, ty) = self.constant_pool.get_name_and_type(name_and_type)?;
                let callee_class = self.constant_pool.resolve_type(class)?;
                let name = self.constant_pool.get_utf8(name)?;

                let (method_index, virtual_index, parameter_count) = *classes
                    .resolve_by_name(callee_class, methods, heap)
                    .virtual_methods
                    .get(name)
                    .ok_or_else(|| MethodError::UnknownStatic(name.to_string()))?;

                self.constant_pool.update_resolved_virtual_method(
                    index,
                    method_index,
                    virtual_index,
                    parameter_count,
                );

                Ok((method_index, parameter_count))
            }
            _ => Err(MethodError::NotStatic(index)),
        }
    }

    pub fn resolve_virtual_method(
        &self,
        index: ConstantPoolIndex,
        classes: &ClassLibrary,
        heap: &mut Heap,
        methods: &MethodTable,
    ) -> Result<(VirtualMethodIndex, usize), MethodError> {
        match self.constant_pool.get_method(index)? {
            MethodReference::ResolvedVirtual {
                virtual_index,
                argument_count,
                ..
            } => Ok((virtual_index, argument_count)),
            MethodReference::Unresolved {
                class,
                name_and_type,
            } => {
                let (name, ty) = self.constant_pool.get_name_and_type(name_and_type)?;
                let callee_class = self.constant_pool.resolve_type(class)?;
                let name = self.constant_pool.get_utf8(name)?;

                let (method_index, virtual_index, parameter_count) = *classes
                    .resolve_by_name(callee_class, methods, heap)
                    .virtual_methods
                    .get(name)
                    .ok_or_else(|| MethodError::UnknownVirtual(name.to_string()))?;

                self.constant_pool.update_resolved_virtual_method(
                    index,
                    method_index,
                    virtual_index,
                    parameter_count,
                );

                Ok((virtual_index, parameter_count))
            }
            _ => Err(MethodError::NotVirtual(index)),
        }
    }

    pub fn get_static_field(&self, info: FieldInfo) -> JvmValue {
        self.static_fields.borrow().get_value(info.offset, info.ty)
    }

    pub fn get_static_field_by_name(
        &self,
        name: &str,
        classes: &ClassLibrary,
    ) -> Result<JvmValue, FieldError> {
        let (class, info) = self.resolve_own_static_field(name, classes)?;
        Ok(classes.resolve(class).get_static_field(info))
    }

    pub fn set_static_field(&self, info: FieldInfo, value: JvmValue) {
        self.static_fields
            .borrow_mut()
            .set_value(info.offset, value);
    }

    /*
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
    */

    /*
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
            (*self.get_method(name)?).clone(),
            parameters,
            self.index(),
            Some(this),
            classes,
            heap,
        )
    }
    */

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
            fields: Fields::init_from_layout(&self.field_layout),
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

    pub fn dispatch_virtual(
        &self,
        method: VirtualMethodIndex,
        classes: &ClassLibrary,
    ) -> MethodIndex {
        classes.resolve(self.class).dispatch_table[method.0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtualMethodIndex(usize);

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

#[derive(thiserror::Error, Debug)]
pub enum MethodError {
    #[error("Unknown instance method '{0}'")]
    UnknownVirtual(String),

    #[error(
        "The method at constant pool index {0} was expected to be virtual, but is not virtual"
    )]
    NotVirtual(ConstantPoolIndex),

    #[error("Unknown static method '{0}'")]
    UnknownStatic(String),

    #[error("The method at constant pool index {0} was expected to be static, but is not static")]
    NotStatic(ConstantPoolIndex),

    #[error(transparent)]
    ConstantPool(#[from] ConstantPoolError),
}

#[derive(thiserror::Error, Debug)]
pub enum ClassCreationError {
    #[error("Failed to resolve the super class")]
    SuperclassResolutionFailed(#[from] ConstantPoolError),
}

#[derive(thiserror::Error, Debug)]
pub enum FieldError {
    #[error(transparent)]
    FieldNotResolvable(#[from] field::FieldError),

    #[error("The static field {0} cannot be resolved")]
    StaticFieldNotFound(String),

    #[error(transparent)]
    ConstantPool(#[from] ConstantPoolError),
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
