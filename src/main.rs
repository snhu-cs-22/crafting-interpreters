use std::process::ExitCode;

use crafting_interpreters::{bytecode, treewalk};
use bytecode::chunk::{Chunk, OpCode};
use bytecode::vm::VM;

fn main() -> ExitCode {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant.into(), 123);
    chunk.write(constant.try_into().unwrap(), 123);

    let constant = chunk.add_constant(3.4);
    chunk.write(OpCode::OpConstant.into(), 123);
    chunk.write(constant.try_into().unwrap(), 123);

    chunk.write(OpCode::OpAdd.into(), 123);

    let constant = chunk.add_constant(5.6);
    chunk.write(OpCode::OpConstant.into(), 123);
    chunk.write(constant.try_into().unwrap(), 123);

    chunk.write(OpCode::OpDivide.into(), 123);
    chunk.write(OpCode::OpNegate.into(), 123);

    chunk.write(OpCode::OpReturn.into(), 123);

    chunk.disassemble("test chunk");
    vm.interpret(chunk);

    ExitCode::SUCCESS
}
