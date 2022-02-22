use dynasmrt::ExecutableBuffer;
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};

pub fn disassemble(buffer: &ExecutableBuffer) -> String {
    let ip = buffer.as_ptr() as u64;
    let mut decoder = Decoder::with_ip(64, buffer, ip, DecoderOptions::NONE);

    let mut formatter = NasmFormatter::new();

    let mut output = String::new();

    let mut instruction = Instruction::default();
    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);
        formatter.format(&instruction, &mut output);
        if decoder.can_decode() {
            output += "\n";
        }
    }

    output
}
