pub mod bytecode;
pub mod class_loader;
pub mod class_parser;
pub mod interpreter;
pub mod model;

use core::slice;
use std::io::{self, Write};

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};
use model::method::Parameters;

use crate::{
    class_loader::BootstrapClassLoader,
    model::{class_library::ClassLibrary, heap::Heap, method::MethodTable},
};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let class_loader = BootstrapClassLoader::new();
    let classes = ClassLibrary::new(class_loader);
    let mut heap = Heap::new();
    let methods = MethodTable::new();

    classes.resolve_by_name("classes/Object", &methods, &mut heap);

    let class = classes
        .resolve_by_name("Test2", &methods, &mut heap)
        .index();

    let (main, _) = classes
        .resolve(class)
        .resolve_own_static_method_by_name("main");
    methods.resolve(main)(&mut heap, &classes, &methods, None, Parameters::empty());

    dbg!(&classes
        .resolve_by_name("Test", &methods, &mut heap)
        .get_static_field_by_name("a", &classes));
    dbg!(&classes
        .resolve_by_name("Test2", &methods, &mut heap)
        .get_static_field_by_name("a", &classes));
    dbg!(&classes
        .resolve_by_name("Test2", &methods, &mut heap)
        .get_static_field_by_name("q", &classes));

    /*
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let string = "Hello World!";

    dynasm!(ops
        ; .arch x64
        ; ->hello:
        ; .bytes string.as_bytes()
    );

    let hello = ops.offset();
    dynasm!(ops
        ; .arch x64
        ; lea rcx, [->hello]
        ; xor edx, edx
        ; mov dl, BYTE string.len() as _
        ; mov rax, QWORD print as _
        ; sub rsp, BYTE 0x28
        ; call rax
        ; add rsp, BYTE 0x28
        ; ret
    );

    let buf = ops.finalize().unwrap();

    let hello_fn: extern "win64" fn() -> bool = unsafe { mem::transmute(buf.ptr(hello)) };

    assert!(hello_fn());
    */
}

pub extern "win64" fn print(buffer: *const u8, length: u64) -> bool {
    io::stdout()
        .write_all(unsafe { slice::from_raw_parts(buffer, length as usize) })
        .is_ok()
}
