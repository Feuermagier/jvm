pub mod bytecode;
pub mod class_loader;
pub mod class_parser;
pub mod interpreter;
pub mod model;

use core::slice;
use std::{
    fs::File,
    io::{self, Read, Write},
    mem,
};

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};
use model::method::Parameters;

use crate::{
    class_loader::BootstrapClassLoader,
    model::{class_library::ClassLibrary, heap::Heap},
};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let class_loader = BootstrapClassLoader::new();
    let classes = ClassLibrary::new(class_loader);
    let mut heap = Heap::new();

    classes.resolve_by_name("classes/Object", &mut heap);

    let class = classes.resolve_by_name("Test", &mut heap).index();

    classes
        .resolve(class)
        .call_static_method("main", Parameters::empty(), &classes, &mut heap)
        .unwrap();

    dbg!(&classes.resolve(class).get_static_field_by_name("y"));

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
