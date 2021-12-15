pub mod bytecode;
pub mod class_loader;
pub mod class_parser;
pub mod interpreter;
pub mod model;
pub mod jit;

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};
use model::method::Parameters;

use crate::{
    class_loader::BootstrapClassLoader,
    model::{class_library::ClassLibrary, heap::Heap, method::MethodTable},
};

#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Test {
    a: u64,
    b: f64,
}

fn main() {
    let test = Test {
        a: 13,
        b: 42.0,
    };

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
        ; sub rsp, 16
        ; mov rax, QWORD -1
        ; mov QWORD [rsp], rax
//        ; push rax
        ; mov rax, QWORD *hello_fn as _
        ; call rax
        ; add rsp, 16
        ; ret
    );

    let caller_buf = ops.finalize().unwrap();
    let caller_fn: extern "sysv64" fn() = unsafe { std::mem::transmute(caller_buf.ptr(caller))};
    caller_fn();

    return;

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
    methods.resolve(main)(main, &mut heap, &classes, &methods, None, Parameters::empty());

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

pub extern "sysv64" fn print(a: u64, b: i64) {
    println!("a: {}, b: {}", a, b);
}