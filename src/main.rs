use std::process::ExitCode;

use crafting_interpreters::{bytecode, treewalk};
use bytecode::chunk::{Chunk, OpCode};

fn main() -> ExitCode {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant.into(), 123);
    chunk.write(constant.try_into().unwrap(), 123);
    chunk.write(OpCode::OpReturn.into(), 123);

    chunk.write(OpCode::OpConstant.into(), 124);
    chunk.write(constant.try_into().unwrap(), 124);
    chunk.write(OpCode::OpReturn.into(), 124);

    chunk.write(OpCode::OpConstant.into(), 124);
    chunk.write(constant.try_into().unwrap(), 124);
    chunk.write(OpCode::OpReturn.into(), 124);

    chunk.write(OpCode::OpConstant.into(), 126);
    chunk.write(constant.try_into().unwrap(), 126);
    chunk.write(OpCode::OpReturn.into(), 126);
    chunk.disassemble("test chunk");

    ExitCode::SUCCESS
}
