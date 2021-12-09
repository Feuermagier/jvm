use std::{cell::RefCell, collections::HashMap};

use appendlist::AppendList;

use crate::{
    class_loader::BootstrapClassLoader,
    class_parser::{self, ParsingError},
    interpreter::ExecutionError,
};

use super::{class::Class, constant_pool::ConstantPoolError, heap::Heap};

pub struct ClassLibrary {
    classes: AppendList<Class>,
    name_mappings: RefCell<HashMap<String, usize>>,
    class_loader: BootstrapClassLoader,
}

impl ClassLibrary {
    pub fn new(class_loader: BootstrapClassLoader) -> Self {
        Self {
            classes: AppendList::new(),
            name_mappings: RefCell::new(HashMap::new()),
            class_loader,
        }
    }

    pub fn resolve_by_name(&self, name: &str, heap: &mut Heap) -> &Class {
        let index = self.name_mappings.borrow().get(name).map(|i| *i);
        if let Some(index) = index {
            &self.classes[index]
        } else {
            let index = self.load(name, heap).unwrap();
            self.resolve(index)
        }
    }

    pub fn resolve(&self, index: ClassIndex) -> &Class {
        &self.classes[index.0]
    }

    /// This function should only be called by a class parser
    pub fn load(&self, name: &str, heap: &mut Heap) -> Result<ClassIndex, ClassResolveError> {
        let index = self.classes.len();

        let bytes = self.class_loader.load_class(name.to_string());
        let (_class_file, mut class) = class_parser::parse(&bytes)?;
        class.update_class_index(ClassIndex(index));

        self.name_mappings
            .borrow_mut()
            .insert(class.name()?.to_string(), index);

        self.classes.push(class);

        self.classes[index].bootstrap(self, heap)?;

        Ok(ClassIndex(index))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ClassIndex(pub usize);

pub const ClassNotLoadedIndex: ClassIndex = ClassIndex(0);

#[derive(thiserror::Error, Debug)]
pub enum ClassResolveError {
    #[error(transparent)]
    ConstantPool(#[from] ConstantPoolError),

    #[error(transparent)]
    ClassParsing(#[from] ParsingError),

    #[error(transparent)]
    ClassInitialization(#[from] ExecutionError),
}