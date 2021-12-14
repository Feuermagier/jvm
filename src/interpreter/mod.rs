pub mod locals;
pub mod stack;
use crate::{
    bytecode,
    interpreter::stack::{StackValue, StackValueWide},
    model::{
        class::{FieldError, MethodError},
        class_library::{ClassIndex, ClassLibrary},
        constant_pool::{ConstantPoolError, ConstantPoolIndex},
        heap::{Heap, HeapIndex},
        method::{Method, MethodTable, Parameters},
        types::TypeError,
        value::{
            JvmDouble, JvmFloat, JvmInt, JvmLong, JvmReference, JvmValue, JVM_EQUAL, JVM_GREATER,
            JVM_LESS,
        },
    },
};

use self::{locals::InterpreterLocals, stack::InterpreterStack};

pub fn execute_method(
    method: Method,
    parameters: Parameters,
    callee_class: ClassIndex,
    this: Option<HeapIndex>,
    classes: &ClassLibrary,
    heap: &mut Heap,
    methods: &MethodTable,
) -> Result<JvmValue, ExecutionError> {
    let callee_class = classes.resolve(callee_class);
    println!(
        "========= Entered method {0} of type {1}",
        &method.name,
        callee_class.name().unwrap()
    );

    let mut locals = InterpreterLocals::new(method.max_locals, parameters, this);
    let mut stack = InterpreterStack::new(method.max_stack);

    let mut pc = 0;

    let code = &method.code;
    let return_value = loop {
        if pc >= code.len() {
            break Err(ExecutionError::MissingReturn);
        }

        let opcode = code[pc];
        //println!("0x{:#04x}", opcode);
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
                stack.push(StackValue::from_int(JvmInt(code[pc + 1] as i32)));
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
                let value = callee_class.get_loadable(index)?;
                stack.push_value(value);
                pc += 1;
            }
            bytecode::LDC_W | bytecode::LDC2_W => {
                let index =
                    ConstantPoolIndex::from(u16::from_be_bytes([code[pc + 1], code[pc + 2]]));
                let value = callee_class.get_loadable(index)?;
                stack.push_value(value);
                pc += 3;
            }

            bytecode::ILOAD
            | bytecode::LLOAD
            | bytecode::FLOAD
            | bytecode::DLOAD
            | bytecode::ALOAD => {
                let index = code[pc + 1];
                stack.push(locals.get(index as usize));
                pc += 2;
            }

            // + ALOAD
            bytecode::ILOAD_0
            | bytecode::LLOAD_0
            | bytecode::FLOAD_0
            | bytecode::DLOAD_0
            | bytecode::ALOAD_0 => {
                stack.push(locals.get(0));
                pc += 1;
            }
            bytecode::ILOAD_1
            | bytecode::LLOAD_1
            | bytecode::FLOAD_1
            | bytecode::DLOAD_1
            | bytecode::ALOAD_1 => {
                stack.push(locals.get(1));
                pc += 1;
            }
            bytecode::ILOAD_2
            | bytecode::LLOAD_2
            | bytecode::FLOAD_2
            | bytecode::DLOAD_2
            | bytecode::ALOAD_2 => {
                stack.push(locals.get(2));
                pc += 1;
            }
            bytecode::ILOAD_3
            | bytecode::LLOAD_3
            | bytecode::FLOAD_3
            | bytecode::DLOAD_3
            | bytecode::ALOAD_3 => {
                stack.push(locals.get(3));
                pc += 1;
            }

            // + array loads
            bytecode::ISTORE
            | bytecode::LSTORE
            | bytecode::FSTORE
            | bytecode::DSTORE
            | bytecode::ASTORE => {
                let index = code[pc + 1];
                locals.set(index as usize, stack.pop());
                pc += 2;
            }

            bytecode::ISTORE_0
            | bytecode::LSTORE_0
            | bytecode::FSTORE_0
            | bytecode::DSTORE_0
            | bytecode::ASTORE_0 => {
                locals.set(0, stack.pop());
                pc += 1;
            }
            bytecode::ISTORE_1
            | bytecode::LSTORE_1
            | bytecode::FSTORE_1
            | bytecode::DSTORE_1
            | bytecode::ASTORE_1 => {
                locals.set(1, stack.pop());
                pc += 1;
            }
            bytecode::ISTORE_2
            | bytecode::LSTORE_2
            | bytecode::FSTORE_2
            | bytecode::DSTORE_2
            | bytecode::ASTORE_2 => {
                locals.set(2, stack.pop());
                pc += 1;
            }
            bytecode::ISTORE_3
            | bytecode::LSTORE_3
            | bytecode::FSTORE_3
            | bytecode::DSTORE_3
            | bytecode::ASTORE_3 => {
                locals.set(3, stack.pop());
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
                locals.set(
                    index,
                    StackValue::from_int(JvmInt(locals.get(index).as_int().0 + increment)),
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
            bytecode::IRETURN => break Ok(JvmValue::Int(stack.pop().as_int())),
            bytecode::LRETURN => break Ok(JvmValue::Long(stack.pop_wide().as_long())),
            bytecode::FRETURN => break Ok(JvmValue::Float(stack.pop().as_float())),
            bytecode::DRETURN => break Ok(JvmValue::Double(stack.pop_wide().as_double())),
            bytecode::ARETURN => break Ok(JvmValue::Reference(stack.pop().as_reference())),
            bytecode::RETURN => break Ok(JvmValue::Void),

            bytecode::GETSTATIC => {
                let (class, field) = callee_class.resolve_static_field(
                    index(code[pc + 1], code[pc + 2]),
                    classes,
                    heap,
                    methods,
                )?;
                let field = classes.resolve(class).get_static_field(field);
                stack.push_value(field);
                pc += 3;
            }
            bytecode::PUTSTATIC => {
                let (class, field) = callee_class.resolve_static_field(
                    index(code[pc + 1], code[pc + 2]),
                    classes,
                    heap,
                    methods,
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
                )?;
                let objectref = stack.pop().as_reference();
                let field = heap.resolve(objectref.to_heap_index()).get_field(field);
                stack.push_value(field);
                pc += 3;
            }
            bytecode::PUTFIELD => {
                let field = callee_class.resolve_instance_field(
                    index(code[pc + 1], code[pc + 2]),
                    classes,
                    heap,
                    methods,
                )?;
                let value = stack.pop_type(field.ty);
                let objectref = stack.pop().as_reference();
                heap.resolve_mut(objectref.to_heap_index())
                    .set_field(field, value);
                pc += 3;
            }

            bytecode::INVOKESPECIAL => {
                let cp_index = index(code[pc + 1], code[pc + 2]);
                //TODO match the signature
                let (method_index, parameter_count) = callee_class
                    .resolve_virtual_method_statically(cp_index, classes, heap, methods)?;
                let instance = stack.pop().as_reference().to_heap_index();
                let parameters = stack.pop_parameters(parameter_count);
                let return_value = methods.resolve(method_index)(
                    heap,
                    classes,
                    methods,
                    Some(instance),
                    parameters,
                );
                stack.push_value(return_value);
                pc += 3;
            }
            bytecode::INVOKESTATIC => {
                let cp_index = index(code[pc + 1], code[pc + 2]);
                let (method_index, parameter_count) =
                    callee_class.resolve_static_method(cp_index, classes, heap, methods)?;
                let parameters = stack.pop_parameters(parameter_count);
                let return_value =
                    methods.resolve(method_index)(heap, classes, methods, None, parameters);
                stack.push_value(return_value);
                pc += 3;
            }
            bytecode::INVOKEVIRTUAL => {
                let cp_index = index(code[pc + 1], code[pc + 2]);
                //TODO match the signature
                let (virtual_index, parameter_count) =
                    callee_class.resolve_virtual_method(cp_index, classes, heap, methods)?;
                let parameters = stack.pop_parameters(parameter_count);
                let instance = stack.pop().as_reference().to_heap_index();
                let method_index = heap
                    .resolve(instance)
                    .dispatch_virtual(virtual_index, classes);
                let return_value = methods.resolve(method_index)(
                    heap,
                    classes,
                    methods,
                    Some(instance),
                    parameters,
                );
                stack.push_value(return_value);
                pc += 3;
            }
            // + invokeinterface, invokedynamic
            bytecode::NEW => {
                let class_name = callee_class.resolve_type(index(code[pc + 1], code[pc + 2]))?;
                let class = classes.resolve_by_name(class_name, methods, heap);
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

#[inline]
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
