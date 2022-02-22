use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap};

use crate::{
    class_parser::ClassData,
    interpreter::{self, ExecutionError},
    jit,
    model::constant_pool::{ConstantPoolEntry, ConstantPoolError},
};

use super::{
    class_library::{ClassIndex, ClassLibrary},
    constant_pool::{ConstantPool, ConstantPoolIndex, FieldReference, MethodReference},
    field::{self, FieldDescriptor, FieldInfo, FieldLayout, Fields},
    heap::Heap,
    method::{MethodCode, MethodData, MethodImplementation, MethodIndex, MethodTable},
    stack::StackPointer,
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

    static_methods: HashMap<String, (MethodIndex, usize)>, // second tuple element is the parameter count
    virtual_methods: HashMap<String, (MethodIndex, VirtualMethodIndex, usize)>, // The MethodIndex is used for static dispatch (i.e. invokespecial)
    dispatch_table: *const MethodIndex,
    dispatch_table_length: usize,
}

impl Class {
    /// Returns (class, statics_length in bytes, dispatch_table_length in dwords)
    pub fn new(
        data: ClassData,
        constant_pool: ConstantPool,
        index: ClassIndex,
        super_class: Option<&Class>,
        methods: &MethodTable,
        static_fields_position: *mut u8,
        dispatch_table_position: *mut MethodIndex,
    ) -> Result<(Self, usize, usize), ClassCreationError> {
        let static_field_layout = field::layout_fields(&FieldLayout::empty(), &data.static_fields);
        let static_fields = unsafe {
            Fields::init_from_layout_at(
                static_fields_position,
                &static_field_layout,
                &data.static_fields,
            )
        };
        let statics_length = static_field_layout.byte_length();

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
            match &desc.code {
                MethodCode::Bytecode(_) => {
                    let method_index = methods.add_method(
                        MethodImplementation::Interpreted,
                        MethodData::from_bytecode_descriptor(desc, index).unwrap(),
                    );
                    static_methods.insert(
                        desc.name.to_string(),
                        (method_index, desc.parameter_count()),
                    );
                }
                MethodCode::Abstract => {
                    panic!("Abstract static method")
                }
                MethodCode::Native => {} // TODO
            }
        }

        let mut virtual_methods = if let Some(super_class) = super_class {
            super_class.virtual_methods.clone()
        } else {
            HashMap::new()
        };
        let mut dispatch_table = Vec::new();
        if let Some(super_class) = super_class {
            unsafe {
                dispatch_table.extend_from_slice(std::slice::from_raw_parts(
                    super_class.dispatch_table,
                    super_class.dispatch_table_length,
                ));
            }
        }
        for desc in &data.methods {
            match &desc.code {
                MethodCode::Bytecode(_) => {
                    let method_index = methods.add_method(
                        MethodImplementation::Interpreted,
                        MethodData::from_bytecode_descriptor(desc, index).unwrap(),
                    );

                    if let Some((old_method_index, virtual_index, _)) =
                        virtual_methods.get_mut(&desc.name)
                    {
                        dispatch_table[virtual_index.0] = method_index;
                        *old_method_index = method_index;
                    } else {
                        let virtual_index = dispatch_table.len();
                        dispatch_table.push(method_index);
                        virtual_methods.insert(
                            desc.name.to_string(),
                            (
                                method_index,
                                VirtualMethodIndex(virtual_index),
                                desc.parameter_count(),
                            ),
                        );
                    }
                }
                MethodCode::Abstract => {} // Abstract method, don't do anything
                MethodCode::Native => {}   // TODO
            }
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                dispatch_table.as_ptr(),
                dispatch_table_position,
                dispatch_table.len(),
            );
        }

        Ok((
            Self {
                index,
                data,
                super_class: super_class.map(|class| class.index()),
                constant_pool,
                static_field_layout,
                static_fields: RefCell::new(static_fields),
                field_layout,
                static_methods,
                virtual_methods,
                dispatch_table: dispatch_table_position,
                dispatch_table_length: dispatch_table.len(),
            },
            statics_length,
            dispatch_table.len(),
        ))
    }

    pub fn update_class_index(&mut self, index: ClassIndex) {
        self.index = index;
    }

    pub fn bootstrap(
        &self,
        methods: &MethodTable,
        classes: &ClassLibrary,
        heap: &mut Heap,
        stack: StackPointer,
    ) -> Result<(), ExecutionError> {
        if let Some((clinit, _)) = self.static_methods.get("<clinit>") {
            let _return_value = interpreter::call_method(*clinit, stack, heap, classes, methods);
        }
        Ok(())
    }

    pub fn resolve_instance_field(
        &self,
        index: ConstantPoolIndex,
        classes: &ClassLibrary,
        heap: &mut Heap,
        methods: &MethodTable,
        stack: StackPointer,
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
                    let callee_class =
                        classes.resolve_by_name(callee_class_name, methods, heap, stack);

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
        stack: StackPointer,
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
                        .resolve_by_name(callee_class_name, methods, heap, stack)
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

    pub fn resolve_static_method(
        &self,
        index: ConstantPoolIndex,
        classes: &ClassLibrary,
        heap: &mut Heap,
        methods: &MethodTable,
        stack: StackPointer,
    ) -> Result<(MethodIndex, usize), MethodError> {
        match self.constant_pool.get_method(index)? {
            MethodReference::ResolvedStatic {
                index,
                parameter_count,
            } => Ok((index, parameter_count)),
            MethodReference::Unresolved {
                class,
                name_and_type,
            } => {
                let (name, ty) = self.constant_pool.get_name_and_type(name_and_type)?;
                let callee_class = self.constant_pool.resolve_type(class)?;
                let name = self.constant_pool.get_utf8(name)?;

                let method = classes
                    .resolve_by_name(callee_class, methods, heap, stack)
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

    /// This does not perform dynamic dispatch!
    pub fn resolve_own_virtual_method_by_name(&self, name: &str) -> (MethodIndex, usize) {
        let (index, _, parameter_count) = *self.virtual_methods.get(name).unwrap();
        return (index, parameter_count)
    }

    pub fn resolve_virtual_method_statically(
        &self,
        index: ConstantPoolIndex,
        classes: &ClassLibrary,
        heap: &mut Heap,
        methods: &MethodTable,
        stack: StackPointer,
    ) -> Result<(MethodIndex, usize), MethodError> {
        match self.constant_pool.get_method(index)? {
            MethodReference::ResolvedStatic {
                index,
                parameter_count,
            } => Ok((index, parameter_count)),
            MethodReference::Unresolved {
                class,
                name_and_type,
            } => {
                let (name, ty) = self.constant_pool.get_name_and_type(name_and_type)?;
                let callee_class = self.constant_pool.resolve_type(class)?;
                let name = self.constant_pool.get_utf8(name)?;

                let (method_index, virtual_index, parameter_count) = *classes
                    .resolve_by_name(callee_class, methods, heap, stack)
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
        stack: StackPointer,
    ) -> Result<(VirtualMethodIndex, usize), MethodError> {
        match self.constant_pool.get_method(index)? {
            MethodReference::ResolvedVirtual {
                virtual_index,
                parameter_count,
                ..
            } => Ok((virtual_index, parameter_count)),
            MethodReference::Unresolved {
                class,
                name_and_type,
            } => {
                let (name, ty) = self.constant_pool.get_name_and_type(name_and_type)?;
                let callee_class = self.constant_pool.resolve_type(class)?;
                let name = self.constant_pool.get_utf8(name)?;

                let (method_index, virtual_index, parameter_count) = *classes
                    .resolve_by_name(callee_class, methods, heap, stack)
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
            .set_value(info.offset, info.ty, value);
    }

    pub fn get_loadable(
        &self,
        index: ConstantPoolIndex,
    ) -> Result<(JvmType, JvmValue), ConstantPoolError> {
        let value = self.constant_pool.get(index)?;
        match value {
            ConstantPoolEntry::Integer(value) => Ok((
                JvmType::Integer,
                JvmValue {
                    int: (*value).into(),
                },
            )),
            ConstantPoolEntry::Long(value) => Ok((
                JvmType::Long,
                JvmValue {
                    long: (*value).into(),
                },
            )),
            ConstantPoolEntry::Float(value) => Ok((
                JvmType::Float,
                JvmValue {
                    float: (*value).into(),
                },
            )),
            ConstantPoolEntry::Double(value) => Ok((
                JvmType::Double,
                JvmValue {
                    double: (*value).into(),
                },
            )),
            ConstantPoolEntry::String(_) => todo!(),
            ConstantPoolEntry::Class { .. } => todo!(),
            // + MethodHandle, MethodType, Dynamic
            _ => Err(ConstantPoolError::NotLoadable(index)),
        }
    }

    pub fn field_layout(&self) -> &FieldLayout {
        &self.field_layout
    }

    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        &self.data.fields
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

    pub fn dispatch_virtual_call(&self, method: VirtualMethodIndex) -> MethodIndex {
        unsafe { *self.dispatch_table.offset(method.0 as isize) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtualMethodIndex(usize);

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
