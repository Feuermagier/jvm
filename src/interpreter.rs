use crate::bytecode::Bytecode;

pub struct Interpreter {

}

impl Interpreter {
    pub fn execute_method(bytecode: &[Bytecode]) {
        for code in bytecode {
            match code {
                Bytecode::ILOAD_1 => todo!(),
                Bytecode::IADD => todo!(),
                Bytecode::IRETURN => todo!(),
                _ => unimplemented!("Unimplemented opcode {:?}", code)
            }
        }
    }
}