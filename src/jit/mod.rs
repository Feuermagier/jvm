mod assembly;

use dynasmrt::{dynasm, relocations::Relocation, Assembler, DynasmApi};

use crate::{
    bytecode,
    model::{
        class_library::ClassLibrary,
        constant_pool::{ConstantPoolIndex, ConstantPoolError},
        heap::{Heap, HeapIndex},
        method::{MethodImplementation, MethodIndex, MethodTable},
        stack::StackValue,
        types::JvmType,
        value::{JvmDouble, JvmFloat, JvmInt, JvmLong, JvmValue},
    },
};

pub fn compile_method(
    method_index: MethodIndex,
    heap: &mut Heap,
    classes: &ClassLibrary,
    methods: &MethodTable,
) -> Result<Box<MethodImplementation>, CompilationError> {
    let method = methods.get_data(method_index);
    let owning_class = classes.resolve(method.owning_class);

    let mut ops = dynasmrt::x64::Assembler::new().unwrap();

    let mut offsets = Vec::with_capacity(method.code.len());

    let mut code_index = 0;
    loop {
        offsets.push(ops.offset());
        if code_index >= method.code.len() {
            return Err(CompilationError::MissingReturn);
        }
        let opcode = method.code[code_index];

        match opcode {
            bytecode::ICONST_M1 => {
                push_constant(&mut ops, StackValue::from_int(JvmInt(-1)));
                code_index += 1;
            }
            bytecode::ICONST_0 => {
                push_constant(&mut ops, StackValue::from_int(JvmInt(0)));
                code_index += 1;
            }
            bytecode::ICONST_1 => {
                push_constant(&mut ops, StackValue::from_int(JvmInt(1)));
                code_index += 1;
            }
            bytecode::ICONST_2 => {
                push_constant(&mut ops, StackValue::from_int(JvmInt(2)));
                code_index += 1;
            }
            bytecode::ICONST_3 => {
                push_constant(&mut ops, StackValue::from_int(JvmInt(3)));
                code_index += 1;
            }
            bytecode::ICONST_4 => {
                push_constant(&mut ops, StackValue::from_int(JvmInt(4)));
                code_index += 1;
            }
            bytecode::ICONST_5 => {
                push_constant(&mut ops, StackValue::from_int(JvmInt(5)));
                code_index += 1;
            }
            bytecode::LCONST_0 => {
                push_wide_constant(&mut ops, StackValue::from_long(JvmLong(0)));
                code_index += 1;
            }
            bytecode::LCONST_1 => {
                push_wide_constant(&mut ops, StackValue::from_long(JvmLong(1)));
                code_index += 1;
            }
            bytecode::FCONST_0 => {
                push_constant(&mut ops, StackValue::from_float(JvmFloat(0.0f32)));
                code_index += 1;
            }
            bytecode::FCONST_1 => {
                push_constant(&mut ops, StackValue::from_float(JvmFloat(1.0f32)));
                code_index += 1;
            }
            bytecode::FCONST_2 => {
                push_constant(&mut ops, StackValue::from_float(JvmFloat(2.0f32)));
                code_index += 1;
            }
            bytecode::DCONST_0 => {
                push_wide_constant(&mut ops, StackValue::from_double(JvmDouble(0.0)));
                code_index += 1;
            }
            bytecode::DCONST_1 => {
                push_wide_constant(&mut ops, StackValue::from_double(JvmDouble(1.0)));
                code_index += 1;
            }

            bytecode::BIPUSH => {
                push_constant(
                    &mut ops,
                    StackValue::from_int(JvmInt(
                        i8::from_be_bytes([method.code[code_index + 1]]) as i32
                    )),
                );
                code_index += 2;
            }
            bytecode::SIPUSH => {
                push_constant(
                    &mut ops,
                    StackValue::from_int(JvmInt(i16::from_be_bytes([
                        method.code[code_index + 1],
                        method.code[code_index + 2],
                    ]) as i32)),
                );
                code_index += 3;
            }

            bytecode::LDC => {
                let index = ConstantPoolIndex::from(method.code[code_index + 1] as u16);
                let (ty, value) = owning_class.get_loadable(index)?;
                push_constant_type(&mut ops, value, ty);
                code_index += 2;
            }
            bytecode::LDC_W | bytecode::LDC2_W => {
                let index =
                    ConstantPoolIndex::from(u16::from_be_bytes([method.code[code_index + 1], method.code[code_index + 2]]));
                let (ty, value) = owning_class.get_loadable(index)?;
                push_constant_type(&mut ops, value, ty);
                code_index += 3;
            }

            _ => todo!("Unimplemented opcode {:#04x}", opcode),
        }
    }
}

fn push_constant<R: Relocation>(ops: &mut Assembler<R>, value: StackValue) {
    dynasm!(ops
        ; .arch x64
        ; mov DWORD [rsi], value.to_raw()
        ; add rsi, 4
    );
}

fn push_wide_constant<R: Relocation>(ops: &mut Assembler<R>, value: (StackValue, StackValue)) {
    dynasm!(ops
        ; .arch x64
        ; mov DWORD [rsi], value.0.to_raw()
        ; mov DWORD [rsi + 4], value.0.to_raw()
        ; add rsi, 8
    );
}

fn push_constant_type<R: Relocation>(ops: &mut Assembler<R>, value: JvmValue, ty: JvmType) {
    match ty {
        JvmType::Integer => push_constant(ops, StackValue::from_int(value.int())),
        JvmType::Long => push_wide_constant(ops, StackValue::from_long(value.long())),
        JvmType::Float => push_constant(ops, StackValue::from_float(value.float())),
        JvmType::Double => push_wide_constant(ops, StackValue::from_double(value.double())),
        JvmType::Reference => push_constant(ops, StackValue::from_reference(value.reference())),
        _ => todo!(),
    }
}

fn push<R: Relocation>(ops: &mut Assembler<R>) {
    dynasm!(ops
        ; .arch x64
        ; mov [rsi], eax
        ; add rsi, 4
    );
}

fn pop<R: Relocation>(ops: &mut Assembler<R>) {
    dynasm!(ops
        ; .arch x64
        ; sub rsi, 4
        ; mov eax, [rsi]
    );
}

fn pop_wide<R: Relocation>(ops: &mut Assembler<R>) {
    dynasm!(ops
        ; .arch x64
        ; sub rsi, 8
        ; mov rax, [rsi]
    );
}

fn load_local<R: Relocation>(ops: &mut Assembler<R>, index: usize) {
    
}

#[derive(Debug, thiserror::Error)]
pub enum CompilationError {
    #[error("The end of the bytecode was reached by no return instruction has been found")]
    MissingReturn,

    #[error(transparent)]
    ConstantPoolError(#[from] ConstantPoolError)
}
