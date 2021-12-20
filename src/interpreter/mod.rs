use crate::{
    bytecode,
    model::{
        class::{FieldError, MethodError},
        class_library::ClassLibrary,
        constant_pool::{ConstantPoolError, ConstantPoolIndex},
        heap::Heap,
        method::{MethodData, MethodIndex, MethodTable},
        stack::{StackFrame, StackPointer, StackValue, StackValueWide},
        types::TypeError,
        value::{
            JvmDouble, JvmFloat, JvmInt, JvmLong, JvmReference, JvmValue, JVM_EQUAL, JVM_GREATER,
            JVM_LESS,
        },
    },
};
use std::arch::{asm, global_asm};

global_asm!(
    ".global interpreter_trampoline",
    "interpreter_trampoline:",
    "sub rsp, 8", // Stack alignment (8B return address to caller of interpreter_trampoline; 16B required => 8B padding)
    "mov rbx, rdi", // Store the global references in registers that are callee-saved in sysv64
    "mov r12, rsi",
    "mov r13, rdx",
    "mov r14, rcx",
    "mov r15, r8",
    "call interpret_method",
    "add rsp, 8", // Restore the global references
    "mov rdi, rbx",
    "mov rsi, r12",
    "mov rdx, r13",
    "mov rcx, r14",
    "mov r8, r15",
    "ret"
);

extern "sysv64" {
    pub fn interpreter_trampoline();
}

pub extern "sysv64" fn call_method(
    method_index: MethodIndex,
    stack: StackPointer,
    heap: &mut Heap,
    classes: &ClassLibrary,
    methods: &MethodTable,
) -> JvmValue {
    unsafe {
        let target = methods.resolve(method_index);
        let heap = heap as *mut Heap;
        let classes = classes as *const ClassLibrary;
        let methods = methods as *const MethodTable;
        let method_index = method_index.into_raw();
        let stack = stack.into_raw();

        let return_value: i64;
        asm!(
            "call {0}",
            "/* {0} */",
            "mov {1}, rax",
            in(reg) target,
            lateout(reg) return_value,
            in("rdi") method_index,
            in("rsi") stack,
            in("rdx") heap,
            in("rcx") classes,
            in("r8") methods,
        );
        JvmValue::from_native(return_value)
    }
}

#[no_mangle]
pub unsafe extern "sysv64" fn interpret_method(
    method_index: MethodIndex,
    stack: StackPointer,
    heap: *mut Heap,
    classes: *const ClassLibrary,
    methods: *const MethodTable,
) -> i64 {
    let heap = &mut *heap;
    let classes = &*classes;
    let methods = &*methods;

    let method = methods.get_data(method_index);
    let mut stack_frame = StackFrame::prepare(stack, method.argument_count, method.max_locals);
    let return_value = interpret(method, heap, classes, methods, &mut stack_frame).unwrap();
    stack_frame.clear();

    return_value.to_native()
}

fn interpret(
    method: &MethodData,
    heap: &mut Heap,
    classes: &ClassLibrary,
    methods: &MethodTable,
    stack: &mut StackFrame,
) -> Result<JvmValue, ExecutionError> {
    let callee_class = classes.resolve(method.owning_class);
    println!(
        "========= Entered method {0} of type {1}",
        &method.name,
        callee_class.name().unwrap()
    );
    let mut pc = 0;
    let code = &method.code;
    let return_value = loop {
        if pc >= code.len() {
            break Err(ExecutionError::MissingReturn);
        }

        let opcode = code[pc];
        //println!("{:#04x}", opcode);
        match opcode {
            bytecode::ICONST_M1 => {
                stack.push(StackValue::from_int(JvmInt(-1)));
                pc += 1;
            }
            bytecode::ICONST_0 => {
                stack.push(StackValue::from_int(JvmInt(0)));
                pc += 1;
            }
            bytecode::ICONST_1 => {
                stack.push(StackValue::from_int(JvmInt(1)));
                pc += 1;
            }
            bytecode::ICONST_2 => {
                stack.push(StackValue::from_int(JvmInt(2)));
                pc += 1;
            }
            bytecode::ICONST_3 => {
                stack.push(StackValue::from_int(JvmInt(3)));
                pc += 1;
            }
            bytecode::ICONST_4 => {
                stack.push(StackValue::from_int(JvmInt(4)));
                pc += 1;
            }
            bytecode::ICONST_5 => {
                stack.push(StackValue::from_int(JvmInt(5)));
                pc += 1;
            }
            bytecode::LCONST_0 => {
                stack.push_wide(StackValue::from_long(JvmLong(0)));
                pc += 1;
            }
            bytecode::LCONST_1 => {
                stack.push_wide(StackValue::from_long(JvmLong(1)));
                pc += 1;
            }
            bytecode::FCONST_0 => {
                stack.push(StackValue::from_float(JvmFloat(0.0f32)));
                pc += 1;
            }
            bytecode::FCONST_1 => {
                stack.push(StackValue::from_float(JvmFloat(1.0f32)));
                pc += 1;
            }
            bytecode::FCONST_2 => {
                stack.push(StackValue::from_float(JvmFloat(2.0f32)));
                pc += 1;
            }
            bytecode::DCONST_0 => {
                stack.push_wide(StackValue::from_double(JvmDouble(0.0)));
                pc += 1;
            }
            bytecode::DCONST_1 => {
                stack.push_wide(StackValue::from_double(JvmDouble(1.0)));
                pc += 1;
            }

            bytecode::BIPUSH => {
                stack.push(StackValue::from_int(JvmInt(
                    i8::from_be_bytes([code[pc + 1]]) as i32,
                )));
                pc += 2;
            }
            bytecode::SIPUSH => {
                stack.push(StackValue::from_int(JvmInt(i16::from_be_bytes([
                    code[pc + 1],
                    code[pc + 2],
                ]) as i32)));
                pc += 3;
            }

            bytecode::LDC => {
                let index = ConstantPoolIndex::from(code[pc + 1] as u16);
                let (ty, value) = callee_class.get_loadable(index)?;
                stack.push_value(value, ty);
                pc += 2;
            }
            bytecode::LDC_W | bytecode::LDC2_W => {
                let index =
                    ConstantPoolIndex::from(u16::from_be_bytes([code[pc + 1], code[pc + 2]]));
                let (ty, value) = callee_class.get_loadable(index)?;
                stack.push_value(value, ty);
                pc += 3;
            }

            bytecode::ILOAD | bytecode::FLOAD | bytecode::ALOAD => {
                let index = code[pc + 1];
                stack.push(stack.get_local(index as usize));
                pc += 2;
            }
            bytecode::LLOAD | bytecode::DLOAD => {
                let index = code[pc + 1] as usize;
                stack.push_wide((stack.get_local(index), stack.get_local(index + 1)));
                pc += 1;
            }
            bytecode::ILOAD_0 | bytecode::FLOAD_0 | bytecode::ALOAD_0 => {
                stack.push(stack.get_local(0));
                pc += 1;
            }
            bytecode::LLOAD_0 | bytecode::DLOAD_0 => {
                stack.push_wide((stack.get_local(0), stack.get_local(1)));
                pc += 1;
            }
            bytecode::ILOAD_1 | bytecode::FLOAD_1 | bytecode::ALOAD_1 => {
                stack.push(stack.get_local(1));
                pc += 1;
            }
            bytecode::LLOAD_1 | bytecode::DLOAD_1 => {
                stack.push_wide((stack.get_local(1), stack.get_local(2)));
                pc += 1;
            }
            bytecode::ILOAD_2 | bytecode::FLOAD_2 | bytecode::ALOAD_2 => {
                stack.push(stack.get_local(2));
                pc += 1;
            }
            bytecode::LLOAD_2 | bytecode::DLOAD_2 => {
                stack.push_wide((stack.get_local(2), stack.get_local(3)));
                pc += 1;
            }
            bytecode::ILOAD_3 | bytecode::FLOAD_3 | bytecode::ALOAD_3 => {
                stack.push(stack.get_local(3));
                pc += 1;
            }
            bytecode::LLOAD_3 | bytecode::DLOAD_3 => {
                stack.push_wide((stack.get_local(3), stack.get_local(4)));
                pc += 1;
            }

            // + array loads
            bytecode::ISTORE | bytecode::FSTORE | bytecode::ASTORE => {
                let index = code[pc + 1];
                let value = stack.pop();
                stack.set_local(index as usize, value);
                pc += 2;
            }
            bytecode::LSTORE | bytecode::DSTORE => {
                let index = code[pc + 1] as usize;
                let top = stack.pop();
                let second = stack.pop();
                stack.set_local(index, second);
                stack.set_local(index + 1, top);
                pc += 2;
            }

            bytecode::ISTORE_0 | bytecode::FSTORE_0 | bytecode::ASTORE_0 => {
                let value = stack.pop();
                stack.set_local(0, value);
                pc += 1;
            }
            bytecode::LSTORE_0 | bytecode::DSTORE_0 => {
                let top = stack.pop();
                let second = stack.pop();
                stack.set_local(0, second);
                stack.set_local(1, top);
                pc += 1;
            }
            bytecode::ISTORE_1 | bytecode::FSTORE_1 | bytecode::ASTORE_1 => {
                let value = stack.pop();
                stack.set_local(1, value);
                pc += 1;
            }
            bytecode::LSTORE_1 | bytecode::DSTORE_1 => {
                let top = stack.pop();
                let second = stack.pop();
                stack.set_local(1, second);
                stack.set_local(2, top);
                pc += 1;
            }
            bytecode::ISTORE_2 | bytecode::FSTORE_2 | bytecode::ASTORE_2 => {
                let value = stack.pop();
                stack.set_local(2, value);
                pc += 1;
            }
            bytecode::LSTORE_2 | bytecode::DSTORE_2 => {
                let top = stack.pop();
                let second = stack.pop();
                stack.set_local(2, second);
                stack.set_local(3, top);
                pc += 1;
            }
            bytecode::ISTORE_3 | bytecode::FSTORE_3 | bytecode::ASTORE_3 => {
                let value = stack.pop();
                stack.set_local(3, value);
                pc += 1;
            }
            bytecode::LSTORE_3 | bytecode::DSTORE_3 => {
                let top = stack.pop();
                let second = stack.pop();
                stack.set_local(3, second);
                stack.set_local(4, top);
                pc += 1;
            }

            // + array stores
            bytecode::POP => {
                stack.pop();
                pc += 1;
            }
            bytecode::POP2 => {
                stack.pop_wide();
                pc += 1;
            }

            bytecode::DUP => {
                let tos = stack.pop();
                stack.push(tos);
                stack.push(tos);
                pc += 1;
            }
            bytecode::DUP_X1 => {
                let top = stack.pop();
                let second = stack.pop();
                stack.push(top);
                stack.push(second);
                stack.push(top);
                pc += 1;
            }
            bytecode::DUP_X2 => {
                let top = stack.pop();
                let second = stack.pop();
                let third = stack.pop();
                stack.push(top);
                stack.push(third);
                stack.push(second);
                stack.push(top);
                pc += 1;
            }
            bytecode::DUP2 => {
                let top = stack.pop();
                let second = stack.pop();
                stack.push(second);
                stack.push(top);
                stack.push(second);
                stack.push(top);
                pc += 1;
            }

            bytecode::SWAP => {
                let top = stack.pop();
                let second = stack.pop();
                stack.push(top);
                stack.push(second);
                pc += 1;
            }

            bytecode::IADD => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                stack.push(StackValue::from_int(JvmInt(op1.0.wrapping_add(op2.0))));
                pc += 1;
            }
            bytecode::LADD => {
                let op2 = stack.pop_wide().as_long();
                let op1 = stack.pop_wide().as_long();
                stack.push_wide(StackValue::from_long(JvmLong(op1.0.wrapping_add(op2.0))));
                pc += 1;
            }
            bytecode::FADD => {
                let op2 = stack.pop().as_float();
                let op1 = stack.pop().as_float();
                stack.push(StackValue::from_float(JvmFloat(op1.0 + op2.0)));
                pc += 1;
            }
            bytecode::DADD => {
                let op2 = stack.pop_wide().as_double();
                let op1 = stack.pop_wide().as_double();
                stack.push_wide(StackValue::from_double(JvmDouble(op1.0 + op2.0)));
                pc += 1;
            }
            bytecode::ISUB => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                stack.push(StackValue::from_int(JvmInt(op1.0.wrapping_sub(op2.0))));
                pc += 1;
            }
            bytecode::LSUB => {
                let op2 = stack.pop_wide().as_long();
                let op1 = stack.pop_wide().as_long();
                stack.push_wide(StackValue::from_long(JvmLong(op1.0.wrapping_sub(op2.0))));
                pc += 1;
            }
            bytecode::FSUB => {
                let op2 = stack.pop().as_float();
                let op1 = stack.pop().as_float();
                stack.push(StackValue::from_float(JvmFloat(op1.0 - op2.0)));
                pc += 1;
            }
            bytecode::DSUB => {
                let op2 = stack.pop_wide().as_double();
                let op1 = stack.pop_wide().as_double();
                stack.push_wide(StackValue::from_double(JvmDouble(op1.0 - op2.0)));
                pc += 1;
            }
            bytecode::IMUL => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                stack.push(StackValue::from_int(JvmInt(op1.0.wrapping_mul(op2.0))));
                pc += 1;
            }
            bytecode::LMUL => {
                let op2 = stack.pop_wide().as_long();
                let op1 = stack.pop_wide().as_long();
                stack.push_wide(StackValue::from_long(JvmLong(op1.0.wrapping_mul(op2.0))));
                pc += 1;
            }
            bytecode::FMUL => {
                let op2 = stack.pop().as_float();
                let op1 = stack.pop().as_float();
                stack.push(StackValue::from_float(JvmFloat(op1.0 * op2.0)));
                pc += 1;
            }
            bytecode::DMUL => {
                let op2 = stack.pop_wide().as_double();
                let op1 = stack.pop_wide().as_double();
                stack.push_wide(StackValue::from_double(JvmDouble(op1.0 * op2.0)));
                pc += 1;
            }
            bytecode::IDIV => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                stack.push(StackValue::from_int(JvmInt(op1.0.wrapping_div(op2.0))));
                pc += 1;
            }
            bytecode::LDIV => {
                let op2 = stack.pop_wide().as_long();
                let op1 = stack.pop_wide().as_long();
                stack.push_wide(StackValue::from_long(JvmLong(op1.0.wrapping_div(op2.0))));
                pc += 1;
            }
            bytecode::FDIV => {
                let op2 = stack.pop().as_float();
                let op1 = stack.pop().as_float();
                stack.push(StackValue::from_float(JvmFloat(op1.0 / op2.0)));
                pc += 1;
            }
            bytecode::DDIV => {
                let op2 = stack.pop_wide().as_double();
                let op1 = stack.pop_wide().as_double();
                stack.push_wide(StackValue::from_double(JvmDouble(op1.0 / op2.0)));
                pc += 1;
            }
            bytecode::IREM => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                stack.push(StackValue::from_int(JvmInt(op1.0 % op2.0)));
                pc += 1;
            }
            bytecode::LREM => {
                let op2 = stack.pop_wide().as_long();
                let op1 = stack.pop_wide().as_long();
                stack.push_wide(StackValue::from_long(JvmLong(op1.0 % op2.0)));
                pc += 1;
            }
            bytecode::FREM => {
                let op2 = stack.pop().as_float();
                let op1 = stack.pop().as_float();
                stack.push(StackValue::from_float(JvmFloat(op1.0 % op2.0)));
                pc += 1;
            }
            bytecode::DREM => {
                let op2 = stack.pop_wide().as_double();
                let op1 = stack.pop_wide().as_double();
                stack.push_wide(StackValue::from_double(JvmDouble(op1.0 % op2.0)));
                pc += 1;
            }
            bytecode::INEG => {
                let op1 = stack.pop().as_int();
                stack.push(StackValue::from_int(JvmInt(-op1.0)));
                pc += 1;
            }
            bytecode::LNEG => {
                let op1 = stack.pop_wide().as_long();
                stack.push_wide(StackValue::from_long(JvmLong(-op1.0)));
                pc += 1;
            }
            bytecode::FNEG => {
                let op1 = stack.pop().as_float();
                stack.push(StackValue::from_float(JvmFloat(-op1.0)));
                pc += 1;
            }
            bytecode::DNEG => {
                let op1 = stack.pop_wide().as_double();
                stack.push_wide(StackValue::from_double(JvmDouble(-op1.0)));
                pc += 1;
            }

            // + Shifts
            bytecode::IAND => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                stack.push(StackValue::from_int(JvmInt(op1.0 & op2.0)));
                pc += 1;
            }
            bytecode::LAND => {
                let op2 = stack.pop_wide().as_long();
                let op1 = stack.pop_wide().as_long();
                stack.push_wide(StackValue::from_long(JvmLong(op1.0 & op2.0)));
                pc += 1;
            }
            bytecode::IOR => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                stack.push(StackValue::from_int(JvmInt(op1.0 | op2.0)));
                pc += 1;
            }
            bytecode::LOR => {
                let op2 = stack.pop_wide().as_long();
                let op1 = stack.pop_wide().as_long();
                stack.push_wide(StackValue::from_long(JvmLong(op1.0 | op2.0)));
                pc += 1;
            }
            bytecode::IXOR => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                stack.push(StackValue::from_int(JvmInt(op1.0 ^ op2.0)));
                pc += 1;
            }
            bytecode::LXOR => {
                let op2 = stack.pop_wide().as_long();
                let op1 = stack.pop_wide().as_long();
                stack.push_wide(StackValue::from_long(JvmLong(op1.0 ^ op2.0)));
                pc += 1;
            }

            bytecode::IINC => {
                let index = code[pc + 1] as usize;
                let increment = unsafe { std::mem::transmute::<u8, i8>(code[pc + 2]) } as i32;
                stack.set_local(
                    index,
                    StackValue::from_int(JvmInt(stack.get_local(index).as_int().0 + increment)),
                );
                pc += 3;
            }

            bytecode::I2L => {
                let value = stack.pop().as_int();
                stack.push_wide(StackValue::from_long(JvmLong(value.0 as i64)));
                pc += 1;
            }
            bytecode::I2F => {
                let value = stack.pop().as_int();
                stack.push(StackValue::from_float(JvmFloat(value.0 as f32)));
                pc += 1;
            }
            bytecode::I2D => {
                let value = stack.pop().as_int();
                stack.push_wide(StackValue::from_double(JvmDouble(value.0 as f64)));
                pc += 1;
            }
            bytecode::L2I => {
                let value = stack.pop_wide().as_long();
                stack.push(StackValue::from_int(JvmInt(value.0 as i32)));
                pc += 1;
            }
            bytecode::L2F => {
                let value = stack.pop_wide().as_long();
                stack.push(StackValue::from_float(JvmFloat(value.0 as f32)));
                pc += 1;
            }
            bytecode::L2D => {
                let value = stack.pop_wide().as_long();
                stack.push_wide(StackValue::from_double(JvmDouble(value.0 as f64)));
                pc += 1;
            }
            bytecode::F2I => {
                let value = stack.pop().as_float();
                stack.push(StackValue::from_int(JvmInt(value.0 as i32)));
                pc += 1;
            }
            bytecode::F2L => {
                let value = stack.pop().as_float();
                stack.push_wide(StackValue::from_long(JvmLong(value.0 as i64)));
                pc += 1;
            }
            bytecode::F2D => {
                let value = stack.pop().as_float();
                stack.push_wide(StackValue::from_double(JvmDouble(value.0 as f64)));
                pc += 1;
            }
            bytecode::D2I => {
                let value = stack.pop_wide().as_double();
                stack.push(StackValue::from_int(JvmInt(value.0 as i32)));
                pc += 1;
            }
            bytecode::D2L => {
                let value = stack.pop_wide().as_double();
                stack.push_wide(StackValue::from_long(JvmLong(value.0 as i64)));
                pc += 1;
            }
            bytecode::D2F => {
                let value = stack.pop_wide().as_double();
                stack.push(StackValue::from_float(JvmFloat(value.0 as f32)));
                pc += 1;
            }
            bytecode::I2B => {
                let value = stack.pop().as_int().0 as i8;
                stack.push(StackValue::from_int(JvmInt(value as i32))); // This does sign-extension
                pc += 1;
            }
            bytecode::I2C => {
                let value = stack.pop().as_int().0 as u8;
                stack.push(StackValue::from_int(JvmInt(value as i32))); //TODO not sure if this does sign-extension (it shouldn't do)
                pc += 1;
            }
            bytecode::I2S => {
                let value = stack.pop().as_int().0 as i16;
                stack.push(StackValue::from_int(JvmInt(value as i32))); // This does sign-extension
                pc += 1;
            }

            bytecode::LCMP => {
                let op2 = stack.pop_wide().as_long();
                let op1 = stack.pop_wide().as_long();
                if op1 > op2 {
                    stack.push(StackValue::from_int(JVM_GREATER));
                } else if op1 == op2 {
                    stack.push(StackValue::from_int(JVM_EQUAL));
                } else {
                    stack.push(StackValue::from_int(JVM_LESS));
                }
                pc += 3;
            }
            bytecode::FCMPG => {
                let op2 = stack.pop().as_float();
                let op1 = stack.pop().as_float();
                if op1.0.is_nan() || op2.0.is_nan() {
                    stack.push(StackValue::from_int(JVM_GREATER));
                } else if op1 > op2 {
                    stack.push(StackValue::from_int(JVM_GREATER));
                } else if op1 == op2 {
                    stack.push(StackValue::from_int(JVM_EQUAL));
                } else {
                    stack.push(StackValue::from_int(JVM_LESS));
                }
                pc += 3;
            }
            bytecode::FCMPL => {
                let op2 = stack.pop().as_float();
                let op1 = stack.pop().as_float();
                if op1.0.is_nan() || op2.0.is_nan() {
                    stack.push(StackValue::from_int(JVM_LESS));
                } else if op1 > op2 {
                    stack.push(StackValue::from_int(JVM_GREATER));
                } else if op1 == op2 {
                    stack.push(StackValue::from_int(JVM_LESS));
                } else {
                    stack.push(StackValue::from_int(JVM_EQUAL));
                }
                pc += 3;
            }
            bytecode::DCMPG => {
                let op2 = stack.pop_wide().as_double();
                let op1 = stack.pop_wide().as_double();
                if op1.0.is_nan() || op2.0.is_nan() {
                    stack.push(StackValue::from_int(JVM_GREATER));
                } else if op1 > op2 {
                    stack.push(StackValue::from_int(JVM_GREATER));
                } else if op1 == op2 {
                    stack.push(StackValue::from_int(JVM_EQUAL));
                } else {
                    stack.push(StackValue::from_int(JVM_LESS));
                }
                pc += 3;
            }
            bytecode::DCMPL => {
                let op2 = stack.pop_wide().as_double();
                let op1 = stack.pop_wide().as_double();
                if op1.0.is_nan() || op2.0.is_nan() {
                    stack.push(StackValue::from_int(JVM_LESS));
                } else if op1 > op2 {
                    stack.push(StackValue::from_int(JVM_GREATER));
                } else if op1 == op2 {
                    stack.push(StackValue::from_int(JVM_EQUAL));
                } else {
                    stack.push(StackValue::from_int(JVM_LESS));
                }
                pc += 3;
            }

            bytecode::IFEQ => {
                let op = stack.pop().as_int();
                if op.0 == 0 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 2;
                }
            }
            bytecode::IFNE => {
                let op = stack.pop().as_int();
                if op.0 != 0 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 2;
                }
            }
            bytecode::IFLT => {
                let op = stack.pop().as_int();
                if op.0 < 0 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 2;
                }
            }
            bytecode::IFGE => {
                let op = stack.pop().as_int();
                if op.0 >= 0 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 2;
                }
            }
            bytecode::IFGT => {
                let op = stack.pop().as_int();
                if op.0 > 0 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 2;
                }
            }
            bytecode::IFLE => {
                let op = stack.pop().as_int();
                if op.0 <= 0 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 2;
                }
            }
            bytecode::IF_ICMPEQ => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                if op1 == op2 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 3;
                }
            }
            bytecode::IF_ICMPNE => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                if op1 != op2 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 3;
                }
            }
            bytecode::IF_ICMPLT => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                if op1 < op2 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 3;
                }
            }
            bytecode::IF_ICMPGE => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                if op1 >= op2 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 3;
                }
            }
            bytecode::IF_ICMPGT => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                if op1 > op2 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 3;
                }
            }
            bytecode::IF_ICMPLE => {
                let op2 = stack.pop().as_int();
                let op1 = stack.pop().as_int();
                if op1 <= op2 {
                    pc = offset(pc, code[pc + 1], code[pc + 2]);
                } else {
                    pc += 3;
                }
            }

            // + IF_ACMPEQ, IF_ACMPNE
            bytecode::GOTO => {
                pc = offset(pc, code[pc + 1], code[pc + 2]);
            }

            // + JSR, RET (maybe)

            // + tableswitch, lookupswitch
            bytecode::IRETURN => {
                break Ok(JvmValue {
                    int: stack.pop().as_int().into(),
                })
            }
            bytecode::LRETURN => {
                break Ok(JvmValue {
                    long: stack.pop_wide().as_long().into(),
                })
            }
            bytecode::FRETURN => {
                break Ok(JvmValue {
                    float: stack.pop().as_float().into(),
                })
            }
            bytecode::DRETURN => {
                break Ok(JvmValue {
                    double: stack.pop_wide().as_double().into(),
                })
            }
            bytecode::ARETURN => {
                break Ok(JvmValue {
                    reference: stack.pop().as_reference().to_heap_index(),
                })
            }
            bytecode::RETURN => break Ok(JvmValue::VOID),

            bytecode::GETSTATIC => {
                let (class, field) = callee_class.resolve_static_field(
                    index(code[pc + 1], code[pc + 2]),
                    classes,
                    heap,
                    methods,
                    stack.get_stack_for_call(),
                )?;
                let value = classes.resolve(class).get_static_field(field);
                stack.push_value(value, field.ty);
                pc += 3;
            }
            bytecode::PUTSTATIC => {
                let (class, field) = callee_class.resolve_static_field(
                    index(code[pc + 1], code[pc + 2]),
                    classes,
                    heap,
                    methods,
                    stack.get_stack_for_call(),
                )?;
                let value = stack.pop_type(field.ty);
                classes.resolve(class).set_static_field(field, value);
                pc += 3;
            }
            bytecode::GETFIELD => {
                let field = callee_class.resolve_instance_field(
                    index(code[pc + 1], code[pc + 2]),
                    classes,
                    heap,
                    methods,
                    stack.get_stack_for_call(),
                )?;
                let objectref = stack.pop().as_reference();
                let value = heap.resolve(objectref.to_heap_index()).get_field(field);
                stack.push_value(value, field.ty);
                pc += 3;
            }
            bytecode::PUTFIELD => {
                let field = callee_class.resolve_instance_field(
                    index(code[pc + 1], code[pc + 2]),
                    classes,
                    heap,
                    methods,
                    stack.get_stack_for_call(),
                )?;
                let value = stack.pop_type(field.ty);
                let objectref = stack.pop().as_reference();
                heap.resolve(objectref.to_heap_index())
                    .set_field(field, value);
                pc += 3;
            }

            bytecode::INVOKESPECIAL => {
                let cp_index = index(code[pc + 1], code[pc + 2]);
                //TODO match the signature
                let (method_index, _) = callee_class.resolve_virtual_method_statically(
                    cp_index,
                    classes,
                    heap,
                    methods,
                    stack.get_stack_for_call(),
                )?;
                let return_type = methods.get_data(method_index).return_type;
                let return_value = call_method(
                    method_index,
                    stack.get_stack_for_call(),
                    heap,
                    classes,
                    methods,
                );
                stack.push_value(return_value, return_type);
                pc += 3;
            }
            bytecode::INVOKESTATIC => {
                let cp_index = index(code[pc + 1], code[pc + 2]);
                let (method_index, _) = callee_class.resolve_static_method(
                    cp_index,
                    classes,
                    heap,
                    methods,
                    stack.get_stack_for_call(),
                )?;
                let return_type = methods.get_data(method_index).return_type;
                let return_value = call_method(
                    method_index,
                    stack.get_stack_for_call(),
                    heap,
                    classes,
                    methods,
                );
                stack.push_value(return_value, return_type);
                pc += 3;
            }
            bytecode::INVOKEVIRTUAL => {
                let cp_index = index(code[pc + 1], code[pc + 2]);
                //TODO match the signature
                let (virtual_index, paramter_count) = callee_class.resolve_virtual_method(
                    cp_index,
                    classes,
                    heap,
                    methods,
                    stack.get_stack_for_call(),
                )?;
                let instance = stack
                    .peek(paramter_count - 1)
                    .as_reference()
                    .to_heap_index();
                let method_index = heap
                    .resolve(instance)
                    .dispatch_virtual(virtual_index, classes);

                let return_type = methods.get_data(method_index).return_type;
                let return_value = call_method(
                    method_index,
                    stack.get_stack_for_call(),
                    heap,
                    classes,
                    methods,
                );
                stack.push_value(return_value, return_type);
                pc += 3;
            }
            // + invokeinterface, invokedynamic
            bytecode::NEW => {
                let class_name = callee_class.resolve_type(index(code[pc + 1], code[pc + 2]))?;
                let class =
                    classes.resolve_by_name(class_name, methods, heap, stack.get_stack_for_call());
                let instance = heap.instantiate(class);
                stack.push(StackValue::from_reference(JvmReference::from_heap_index(
                    instance,
                )));
                pc += 3;
            }

            _ => todo!("Unimplemented opcode {:#04x}", opcode),
        }
    };
    println!(
        "========= Exited method {0} of type {1}",
        &method.name,
        callee_class.name().unwrap()
    );
    return_value
}

#[inline(always)]
fn offset(pc: usize, byte1: u8, byte2: u8) -> usize {
    //hack
    // Should work because of the two complement's representation of i16 and the wrapping add
    // as long as no overflows occur (we trust the class file)
    pc.wrapping_add(i16::from_be_bytes([byte1, byte2]) as usize)
}

#[inline]
fn index(byte1: u8, byte2: u8) -> ConstantPoolIndex {
    u16::from_be_bytes([byte1, byte2]).into()
}

#[derive(thiserror::Error, Debug)]
pub enum ExecutionError {
    #[error("last instruction was not a return instruction")]
    MissingReturn,

    #[error("method error")]
    MethodError {
        #[from]
        value: MethodError,
    },

    #[error("type error")]
    TypeError {
        #[from]
        value: TypeError,
    },

    #[error("constant pool error")]
    ConstantPoolError {
        #[from]
        value: ConstantPoolError,
    },

    #[error("field error")]
    FieldError {
        #[from]
        value: FieldError,
    },
}
