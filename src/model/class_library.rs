use std::{cell::RefCell, collections::HashMap};

use appendlist::AppendList;

use crate::{
    class_loader::BootstrapClassLoader,
    class_parser::{self, ParsingError},
    interpreter::ExecutionError,
    list::NativeList,
};

use super::{
    class::{Class, ClassCreationError},
    constant_pool::ConstantPoolError,
    heap::Heap,
    method::{MethodIndex, MethodTable},
    stack::StackPointer,
};

#[repr(C)]
pub struct ClassLibrary {
    dispatch_tables: NativeList<MethodIndex>,
    static_attributes: NativeList<u8>,
    dispatch_table_tail: RefCell<usize>, // In dwords (u32), i.e. size_of<MethodIndex>()
    statics_tail: RefCell<usize>,        // In bytes
    classes: AppendList<Class>,
    name_mappings: RefCell<HashMap<String, usize>>,
    class_loader: BootstrapClassLoader,
}

impl ClassLibrary {
    pub fn new(class_loader: BootstrapClassLoader) -> Self {
        Self {
            dispatch_tables: NativeList::alloc(1000, 8),
            static_attributes: NativeList::alloc(4000, 8),
            classes: AppendList::new(),
            name_mappings: RefCell::new(HashMap::new()),
            class_loader,
            dispatch_table_tail: RefCell::new(0),
            statics_tail: RefCell::new(0),
        }
    }

    pub fn resolve_by_name(
        &self,
        name: &str,
        methods: &MethodTable,
        heap: &mut Heap,
        stack: StackPointer,
    ) -> &Class {
        let index = self.name_mappings.borrow().get(name).map(|i| *i);
        if let Some(index) = index {
            &self.classes[index]
        } else {
            let index = self.load(name, heap, methods, stack).unwrap();
            self.resolve(index)
        }
    }

    pub fn resolve(&self, index: ClassIndex) -> &Class {
        &self.classes[index.0]
    }

    /// This function should only be called by a class parser
    pub fn load(
        &self,
        name: &str,
        heap: &mut Heap,
        methods: &MethodTable,
        stack: StackPointer,
    ) -> Result<ClassIndex, ClassResolveError> {
        log::info!("Loading class {}", name);
        let bytes = self.class_loader.load_class(name.to_string());
        let (_file, data, constant_pool) = class_parser::parse(&bytes)?;

        let super_class = if data.super_class.is_valid() {
            let name = constant_pool.resolve_type(data.super_class)?;
            Some(self.resolve_by_name(name, methods, heap, stack))
        } else {
            None
        };

        // The following code for creating and updating the class must not be interrupted by an access to the ClassLibrary
        // or the indices will be wrong
        let index = self.classes.len();
        let statics_position = unsafe {
            self.static_attributes
                .get_pointer()
                .offset(*self.statics_tail.borrow() as isize)
        };
        let dispatch_table_position = unsafe {
            self.dispatch_tables
                .get_pointer()
                .offset(*self.dispatch_table_tail.borrow() as isize)
        };
        let (class, statics_length, dispatch_table_length) = Class::new(
            data,
            constant_pool,
            ClassIndex(index),
            super_class,
            methods,
            statics_position,
            dispatch_table_position,
        )?;
        *self.statics_tail.borrow_mut() += statics_length;
        *self.dispatch_table_tail.borrow_mut() += dispatch_table_length;
        self.name_mappings
            .borrow_mut()
            .insert(class.name()?.to_string(), index);
        self.classes.push(class);

        self.classes[index].bootstrap(methods, self, heap, stack)?;

        Ok(ClassIndex(index))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ClassIndex(pub usize);

#[derive(thiserror::Error, Debug)]
pub enum ClassResolveError {
    #[error(transparent)]
    ConstantPool(#[from] ConstantPoolError),

    #[error(transparent)]
    ClassParsing(#[from] ParsingError),

    #[error(transparent)]
    ClassCreation(#[from] ClassCreationError),

    #[error(transparent)]
    ClassInitialization(#[from] ExecutionError),
}
