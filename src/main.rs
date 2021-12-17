#![feature(asm_sym)]
#![feature(naked_functions)]
#![feature(int_roundings)]

pub mod bytecode;
pub mod class_loader;
pub mod class_parser;
pub mod interpreter;
pub mod jit;
pub mod model;

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};
use std::arch::{asm, global_asm};

use crate::{
    class_loader::BootstrapClassLoader,
    model::{
        class_library::ClassLibrary,
        heap::{Heap, NULL_POINTER},
        method::MethodTable,
        stack::StackPointer,
    },
};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let class_loader = BootstrapClassLoader::new();
    let classes = ClassLibrary::new(class_loader);
    let mut heap = Heap::new();
    let methods = MethodTable::new(100);
    let stack = StackPointer::with_size(20000);

    classes.resolve_by_name("classes/Object", &methods, &mut heap, stack);

    let class = classes.resolve_by_name("Test", &methods, &mut heap, stack).index();

    let (main, _) = classes
        .resolve(class)
        .resolve_own_static_method_by_name("main");
    interpreter::call_method(main, stack, &mut heap, &classes, &methods);

    dbg!(&classes
        .resolve_by_name("Test", &methods, &mut heap, stack)
        .get_static_field_by_name("a", &classes).unwrap().long());

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

/*
global_asm!(
    ".global interpreter_trampoline",
    "interpreter_trampoline:",
    "mov rsi, qword ptr [rsp+8]",
    "mov rdi, 5",
    "call print",
    "ret"
);

extern "sysv64" {
    fn interpreter_trampoline();
}


#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Test {
    a: u64,
    b: f64,
}


#[no_mangle]
pub extern "sysv64" fn print(a: u64, b: i64) {
    println!("a: {}, b: {}", a, b);
}


fn test() {
    let test = Test { a: 13, b: 42.0 };

    let mut ops = dynasmrt::x64::Assembler::new().unwrap();

    dynasm!(ops
        ; .arch x64
        ; ->test:
        ; .bytes bytemuck::bytes_of(&test)
    );
    let hello = ops.offset();
    dynasm!(ops
            ; .arch x64
    //        ; push rbp
    //        ; mov rbp, rsp
            ; mov rsi, QWORD [rsp + 8]
            ; lea rdi, [->test]
            ; add rdi, BYTE bytemuck::offset_of!(test, Test, a) as i8
            ; mov rdi, QWORD [rdi]
            ; mov rax, QWORD print as _
            ; call rax
    //        ; pop rbp
            ; ret
        );

    let buf = ops.finalize().unwrap();

    let hello_fn: extern "sysv64" fn() = unsafe { std::mem::transmute(buf.ptr(hello)) };
    let hello_fn = Box::new(hello_fn);

    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let caller = ops.offset();

    dynasm!(ops
                ; .arch x64
                ; sub rsp, 16 // Stack must be aligned at 16 byte boundaries
                ; mov rax, QWORD -1
                ; mov QWORD [rsp], rax
        //        ; push rax
    //            ; mov rax, QWORD *hello_fn as _
                ; mov rax, QWORD interpreter_trampoline as _
                ; call rax
                ; add rsp, 16
                ; ret
            );

    let caller_buf = ops.finalize().unwrap();
    let caller_fn: extern "sysv64" fn() = unsafe { std::mem::transmute(caller_buf.ptr(caller)) };
    caller_fn();
}
*/
