mod assembly;

use dynasmrt::DynasmApi;

use crate::model::{
    class_library::ClassLibrary,
    heap::{HeapIndex, Heap},
    method::{MethodImplementation, MethodIndex, MethodTable},
};

/*
pub fn compile_method(
    method_index: MethodIndex,
    heap: &mut Heap,
    classes: &ClassLibrary,
    methods: &MethodTable,
) -> Box<MethodImplementation> {

    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let method_start = ops.offset();
    
}
*/