pub mod locals;
pub mod stack;
use crate::{
    bytecode,
    model::{
        class::{Class, FieldError, LoadedClasses, MethodError},
        constant_pool::{ConstantPoolError, ConstantPoolIndex},
        heap::{Heap, HeapIndex},
        method::{Method, Parameters},
        types::TypeError,
        value::JvmValue,
    },
};

use self::{locals::InterpreterLocals, stack::InterpreterStack};

pub fn execute_method(
    method: &Method,
    parameters: Parameters,
    callee_class: &Class,
    this: Option<HeapIndex>,
    classes: &LoadedClasses,
    heap: &mut Heap,
) -> Result<JvmValue, ExecutionError> {
    let mut locals = InterpreterLocals::new(
        method.max_locals,
        parameters,
        this.map(|this| JvmValue::Reference(this)),
    );
    let mut stack = InterpreterStack::new(method.max_stack);

    let mut pc = 0;

    let code = &method.code;
    loop {
        if pc >= code.len() {
            break Err(ExecutionError::MissingReturn);
        }
        let opcode = code[pc];
        println!("{:#04x}", opcode);
        match opcode {
            bytecode::ICONST_M1 => {
                stack.push(JvmValue::Int(-1));
                pc += 1;
            }
            bytecode::ICONST_0 => {
                stack.push(JvmValue::Int(0));
                pc += 1;
            }
            bytecode::ICONST_1 => {
                stack.push(JvmValue::Int(1));
                pc += 1;
            }
            bytecode::ICONST_2 => {
                stack.push(JvmValue::Int(2));
                pc += 1;
            }
            bytecode::ICONST_3 => {
                stack.push(JvmValue::Int(3));
                pc += 1;
            }
            bytecode::ICONST_4 => {
                stack.push(JvmValue::Int(4));
                pc += 1;
            }
            bytecode::ICONST_5 => {
                stack.push(JvmValue::Int(5));
                pc += 1;
            }
            bytecode::LCONST_0 => {
                stack.push(JvmValue::Long(0));
                pc += 1;
            }
            bytecode::LCONST_1 => {
                stack.push(JvmValue::Long(1));
                pc += 1;
            }
            bytecode::FCONST_0 => {
                stack.push(JvmValue::Float(0.0));
                pc += 1;
            }
            bytecode::FCONST_1 => {
                stack.push(JvmValue::Float(1.0));
                pc += 1;
            }
            bytecode::FCONST_2 => {
                stack.push(JvmValue::Float(2.0));
                pc += 1;
            }
            bytecode::DCONST_0 => {
                stack.push(JvmValue::Double(0.0));
                pc += 1;
            }
            bytecode::DCONST_1 => {
                stack.push(JvmValue::Double(1.0));
                pc += 1;
            }

            bytecode::BIPUSH => {
                stack.push(JvmValue::Int(code[pc + 1] as i32));
                pc += 2;
            }
            bytecode::SIPUSH => {
                stack.push(JvmValue::Int(
                    i16::from_be_bytes([code[pc + 1], code[pc + 2]]) as i32,
                ));
                pc += 3;
            }

            bytecode::LDC => {
                let index = ConstantPoolIndex::from(code[pc + 1] as u16);
                let value = callee_class.get_loadable(index)?;
                stack.push(value);
                pc += 1;
            }
            bytecode::LDC_W | bytecode::LDC2_W => {
                let index =
                    ConstantPoolIndex::from(u16::from_be_bytes([code[pc + 1], code[pc + 2]]));
                let value = callee_class.get_loadable(index)?;
                stack.push(value);
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
                let tos = stack.pop();
                match tos {
                    JvmValue::Long(_) | JvmValue::Double(_) => {}
                    _ => {
                        stack.pop();
                    }
                }
                pc += 1;
            }

            bytecode::DUP => {
                let tos = stack.pop();
                stack.push(tos.clone());
                stack.push(tos);
                pc += 1;
            }
            bytecode::DUP_X1 => {
                let top = stack.pop();
                let second = stack.pop();
                stack.push(top.clone());
                stack.push(second);
                stack.push(top);
                pc += 1;
            }
            bytecode::DUP_X2 => {
                let top = stack.pop();
                let second = stack.pop();
                let third = stack.pop();
                stack.push(top.clone());
                stack.push(third);
                stack.push(second);
                stack.push(top);
                pc += 1;
            }
            bytecode::DUP2 => {
                let top = stack.pop();
                match top {
                    JvmValue::Double(_) | JvmValue::Long(_) => {
                        stack.push(top.clone());
                        stack.push(top);
                    }
                    _ => {
                        let second = stack.pop();
                        stack.push(second.clone());
                        stack.push(top.clone());
                        stack.push(second);
                        stack.push(top);
                    }
                }
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
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                stack.push(JvmValue::Int(op1.wrapping_add(op2)));
                pc += 1;
            }
            bytecode::LADD => {
                let op2 = stack.pop().as_long()?;
                let op1 = stack.pop().as_long()?;
                stack.push(JvmValue::Long(op1.wrapping_add(op2)));
                pc += 1;
            }
            bytecode::FADD => {
                let op2 = stack.pop().as_float()?;
                let op1 = stack.pop().as_float()?;
                stack.push(JvmValue::Float(op1 + op2));
                pc += 1;
            }
            bytecode::DADD => {
                let op2 = stack.pop().as_double()?;
                let op1 = stack.pop().as_double()?;
                stack.push(JvmValue::Double(op1 + op2));
                pc += 1;
            }
            bytecode::ISUB => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                stack.push(JvmValue::Int(op1.wrapping_sub(op2)));
                pc += 1;
            }
            bytecode::LSUB => {
                let op2 = stack.pop().as_long()?;
                let op1 = stack.pop().as_long()?;
                stack.push(JvmValue::Long(op1.wrapping_sub(op2)));
                pc += 1;
            }
            bytecode::FSUB => {
                let op2 = stack.pop().as_float()?;
                let op1 = stack.pop().as_float()?;
                stack.push(JvmValue::Float(op1 - op2));
                pc += 1;
            }
            bytecode::DSUB => {
                let op2 = stack.pop().as_double()?;
                let op1 = stack.pop().as_double()?;
                stack.push(JvmValue::Double(op1 - op2));
                pc += 1;
            }
            bytecode::IMUL => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                stack.push(JvmValue::Int(op1.wrapping_mul(op2)));
                pc += 1;
            }
            bytecode::LMUL => {
                let op2 = stack.pop().as_long()?;
                let op1 = stack.pop().as_long()?;
                stack.push(JvmValue::Long(op1.wrapping_mul(op2)));
                pc += 1;
            }
            bytecode::FMUL => {
                let op2 = stack.pop().as_float()?;
                let op1 = stack.pop().as_float()?;
                stack.push(JvmValue::Float(op1 * op2));
                pc += 1;
            }
            bytecode::DMUL => {
                let op2 = stack.pop().as_double()?;
                let op1 = stack.pop().as_double()?;
                stack.push(JvmValue::Double(op1 * op2));
                pc += 1;
            }
            bytecode::IDIV => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                stack.push(JvmValue::Int(op1.wrapping_div(op2)));
                pc += 1;
            }
            bytecode::LDIV => {
                let op2 = stack.pop().as_long()?;
                let op1 = stack.pop().as_long()?;
                stack.push(JvmValue::Long(op1.wrapping_div(op2)));
                pc += 1;
            }
            bytecode::FDIV => {
                let op2 = stack.pop().as_float()?;
                let op1 = stack.pop().as_float()?;
                stack.push(JvmValue::Float(op1 / op2));
                pc += 1;
            }
            bytecode::DDIV => {
                let op2 = stack.pop().as_double()?;
                let op1 = stack.pop().as_double()?;
                stack.push(JvmValue::Double(op1 / op2));
                pc += 1;
            }
            bytecode::IREM => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                stack.push(JvmValue::Int(op1 % op2));
                pc += 1;
            }
            bytecode::LREM => {
                let op2 = stack.pop().as_long()?;
                let op1 = stack.pop().as_long()?;
                stack.push(JvmValue::Long(op1 % op2));
                pc += 1;
            }
            bytecode::FREM => {
                let op2 = stack.pop().as_float()?;
                let op1 = stack.pop().as_float()?;
                stack.push(JvmValue::Float(op1 % op2));
                pc += 1;
            }
            bytecode::DREM => {
                let op2 = stack.pop().as_double()?;
                let op1 = stack.pop().as_double()?;
                stack.push(JvmValue::Double(op1 % op2));
                pc += 1;
            }
            bytecode::INEG => {
                let op1 = stack.pop().as_int()?;
                stack.push(JvmValue::Int(-op1));
                pc += 1;
            }
            bytecode::LNEG => {
                let op1 = stack.pop().as_long()?;
                stack.push(JvmValue::Long(-op1));
                pc += 1;
            }
            bytecode::FNEG => {
                let op1 = stack.pop().as_float()?;
                stack.push(JvmValue::Float(-op1));
                pc += 1;
            }
            bytecode::DNEG => {
                let op1 = stack.pop().as_double()?;
                stack.push(JvmValue::Double(-op1));
                pc += 1;
            }

            // + Shifts
            bytecode::IAND => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                stack.push(JvmValue::Int(op1 & op2));
                pc += 1;
            }
            bytecode::LAND => {
                let op2 = stack.pop().as_long()?;
                let op1 = stack.pop().as_long()?;
                stack.push(JvmValue::Long(op1 & op2));
                pc += 1;
            }
            bytecode::IOR => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                stack.push(JvmValue::Int(op1 | op2));
                pc += 1;
            }
            bytecode::LOR => {
                let op2 = stack.pop().as_long()?;
                let op1 = stack.pop().as_long()?;
                stack.push(JvmValue::Long(op1 | op2));
                pc += 1;
            }
            bytecode::IXOR => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                stack.push(JvmValue::Int(op1 ^ op2));
                pc += 1;
            }
            bytecode::LXOR => {
                let op2 = stack.pop().as_long()?;
                let op1 = stack.pop().as_long()?;
                stack.push(JvmValue::Long(op1 ^ op2));
                pc += 1;
            }

            bytecode::IINC => {
                let index = code[pc + 1] as usize;
                let increment = unsafe { std::mem::transmute::<u8, i8>(code[pc + 2]) } as i32;
                locals.set(
                    index,
                    JvmValue::Int(locals.get(index).as_int()? + increment),
                );
                pc += 3;
            }

            bytecode::I2L => {
                let value = stack.pop().as_int()?;
                stack.push(JvmValue::Long(value as i64));
                pc += 1;
            }
            bytecode::I2F => {
                let value = stack.pop().as_int()?;
                stack.push(JvmValue::Float(value as f32));
                pc += 1;
            }
            bytecode::I2D => {
                let value = stack.pop().as_int()?;
                stack.push(JvmValue::Double(value as f64));
                pc += 1;
            }
            bytecode::L2I => {
                let value = stack.pop().as_long()?;
                stack.push(JvmValue::Int(value as i32));
                pc += 1;
            }
            bytecode::L2F => {
                let value = stack.pop().as_long()?;
                stack.push(JvmValue::Float(value as f32));
                pc += 1;
            }
            bytecode::L2D => {
                let value = stack.pop().as_long()?;
                stack.push(JvmValue::Double(value as f64));
                pc += 1;
            }
            bytecode::F2I => {
                let value = stack.pop().as_float()?;
                stack.push(JvmValue::Int(value as i32));
                pc += 1;
            }
            bytecode::F2L => {
                let value = stack.pop().as_float()?;
                stack.push(JvmValue::Long(value as i64));
                pc += 1;
            }
            bytecode::F2D => {
                let value = stack.pop().as_float()?;
                stack.push(JvmValue::Double(value as f64));
                pc += 1;
            }
            bytecode::D2I => {
                let value = stack.pop().as_double()?;
                stack.push(JvmValue::Int(value as i32));
                pc += 1;
            }
            bytecode::D2L => {
                let value = stack.pop().as_double()?;
                stack.push(JvmValue::Long(value as i64));
                pc += 1;
            }
            bytecode::D2F => {
                let value = stack.pop().as_float()?;
                stack.push(JvmValue::Float(value as f32));
                pc += 1;
            }
            bytecode::I2B => {
                let value = stack.pop().as_int()? as i8;
                stack.push(JvmValue::Int(value as i32)); // This does sign-extension
                pc += 1;
            }
            bytecode::I2C => {
                let value = stack.pop().as_int()? as u8;
                stack.push(JvmValue::Int(value as i32)); //TODO not sure if this does sign-extension (it shouldn't do)
                pc += 1;
            }
            bytecode::I2S => {
                let value = stack.pop().as_int()? as i16;
                stack.push(JvmValue::Int(value as i32)); // This does sign-extension
                pc += 1;
            }

            bytecode::LCMP => {
                let op2 = stack.pop().as_long()?;
                let op1 = stack.pop().as_long()?;
                if op1 > op2 {
                    stack.push(JvmValue::Int(1));
                } else if op1 == op2 {
                    stack.push(JvmValue::Int(-1));
                } else {
                    stack.push(JvmValue::Int(0));
                }
                pc += 3;
            }
            bytecode::FCMPG => {
                let op2 = stack.pop().as_float()?;
                let op1 = stack.pop().as_float()?;
                if op1.is_nan() || op2.is_nan() {
                    stack.push(JvmValue::Int(1));
                } else if op1 > op2 {
                    stack.push(JvmValue::Int(1));
                } else if op1 == op2 {
                    stack.push(JvmValue::Int(-1));
                } else {
                    stack.push(JvmValue::Int(0));
                }
                pc += 3;
            }
            bytecode::FCMPL => {
                let op2 = stack.pop().as_float()?;
                let op1 = stack.pop().as_float()?;
                if op1.is_nan() || op2.is_nan() {
                    stack.push(JvmValue::Int(-1));
                } else if op1 > op2 {
                    stack.push(JvmValue::Int(1));
                } else if op1 == op2 {
                    stack.push(JvmValue::Int(-1));
                } else {
                    stack.push(JvmValue::Int(0));
                }
                pc += 3;
            }
            bytecode::DCMPG => {
                let op2 = stack.pop().as_double()?;
                let op1 = stack.pop().as_double()?;
                if op1.is_nan() || op2.is_nan() {
                    stack.push(JvmValue::Int(1));
                } else if op1 > op2 {
                    stack.push(JvmValue::Int(1));
                } else if op1 == op2 {
                    stack.push(JvmValue::Int(-1));
                } else {
                    stack.push(JvmValue::Int(0));
                }
                pc += 3;
            }
            bytecode::DCMPL => {
                let op2 = stack.pop().as_double()?;
                let op1 = stack.pop().as_double()?;
                if op1.is_nan() || op2.is_nan() {
                    stack.push(JvmValue::Int(-1));
                } else if op1 > op2 {
                    stack.push(JvmValue::Int(1));
                } else if op1 == op2 {
                    stack.push(JvmValue::Int(-1));
                } else {
                    stack.push(JvmValue::Int(0));
                }
                pc += 3;
            }

            bytecode::IFEQ => {
                let op = stack.pop().as_int()?;
                if op == 0 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IFNE => {
                let op = stack.pop().as_int()?;
                if op != 0 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IFLT => {
                let op = stack.pop().as_int()?;
                if op < 0 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IFGE => {
                let op = stack.pop().as_int()?;
                if op >= 0 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IFGT => {
                let op = stack.pop().as_int()?;
                if op > 0 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IFLE => {
                let op = stack.pop().as_int()?;
                if op <= 0 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IF_ICMPEQ => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                if op1 == op2 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IF_ICMPNE => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                if op1 != op2 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IF_ICMPLT => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                if op1 < op2 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IF_ICMPGE => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                if op1 >= op2 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IF_ICMPGT => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                if op1 > op2 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }
            bytecode::IF_ICMPLE => {
                let op2 = stack.pop().as_int()?;
                let op1 = stack.pop().as_int()?;
                if op1 <= op2 {
                    pc = jump(code[pc + 1], code[pc + 2]);
                }
            }

            // + IF_ACMPEQ, IF_ACMPNE
            bytecode::GOTO => {
                pc = jump(code[pc + 1], code[pc + 2]);
            }

            // + JSR, RET (maybe)

            // + tableswitch, lookupswitch
            bytecode::IRETURN
            | bytecode::LRETURN
            | bytecode::FRETURN
            | bytecode::DRETURN
            | bytecode::ARETURN => break Ok(stack.pop()),
            bytecode::RETURN => break Ok(JvmValue::Void),

            bytecode::GETSTATIC => {
                let field_name = callee_class.resolve_field(index(code[pc + 1], code[pc + 2]))?;
                let field = callee_class.get_static_field(field_name)?;
                stack.push(field);
                pc += 3;
            }
            bytecode::PUTSTATIC => {
                let field_name = callee_class.resolve_field(index(code[pc + 1], code[pc + 2]))?;
                callee_class.set_static_field(field_name, stack.pop())?;
                pc += 3;
            }
            bytecode::GETFIELD => {
                let field_name = callee_class.resolve_field(index(code[pc + 1], code[pc + 2]))?;
                let field = heap
                    .resolve(this.expect("Getting an instance field but this is not bound"))
                    .get_field(field_name, classes)?;
                stack.push(field);
                pc += 3;
            }
            bytecode::PUTFIELD => {
                let field_name = callee_class.resolve_field(index(code[pc + 1], code[pc + 2]))?;
                heap.resolve(this.expect("Getting an instance field but this is not bound"))
                    .set_field(field_name, classes, stack.pop())?;
                pc += 3;
            }

            bytecode::INVOKESPECIAL => {
                let method_index = index(code[pc + 1], code[pc + 2]);
                let (class, name, ty) = callee_class.resolve_method(method_index)?;
                let class = classes.resolve_by_name(class);
                class.call_method(
                    name,
                    Parameters::empty(),
                    stack.pop().as_reference()?,
                    classes,
                    heap,
                )?;
                pc += 3;
            }
            // + invokestatic, invokevirtual, invokeinterface, invokedynamic
            bytecode::NEW => {
                let class_name = callee_class.resolve_type(index(code[pc + 1], code[pc + 2]))?;
                let instance = heap.instantiate(classes.resolve_by_name(class_name));
                stack.push(JvmValue::Reference(instance));
                pc += 3;
            }

            _ => todo!("Unimplemented opcode {:#04x}", opcode),
        }
    }
}

fn jump(byte1: u8, byte2: u8) -> usize {
    u16::from_le_bytes([byte1, byte2]) as usize
}

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
