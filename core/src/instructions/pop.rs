//!POP instruction
//! Pops a value off the stack.

use std::fmt;

use crate::cpu::CPU;
use crate::decoder::{Instruction, Operand};
use crate::memory::Memory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidOperand,
    MemoryError(crate::memory::MemoryError),
    StackOverflow,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidOperand => write!(f, "Invalid operand for POP"),
            ExecutionError::MemoryError(e) => write!(f, "Memory error: {}", e),
            ExecutionError::StackOverflow => write!(f, "Stack overflow occurred"),
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<crate::memory::MemoryError> for ExecutionError {
    fn from(err: crate::memory::MemoryError) -> Self {
        ExecutionError::MemoryError(err)
    }
}

//Execute POP instruction
//
//pops a value off the stack
//
//1. Read 4 bytes from [ESP]
//2. Increment ESP by 4 (pop moves esp up)
//3. write value into destination register:
//     if dest is register, write directly
//    if dest is memory, write 4 bytes to memory loc
//4. Advance EIP to next instruction
//
//**notes
//flags not affected
//assumes 32-bit operands
//
//Arguments:
//* `cpu` - Mutable reference to CPU state
//* `memory` - Mutable reference to memory system
//* `instruction` - The decoded POP instruction

pub fn execute(
    cpu: &mut CPU,
    memory: &mut Memory,
    instruction: &Instruction,
) -> Result<(), ExecutionError> {
    //destination operand
    let dest_operand = instruction.dest.ok_or(ExecutionError::InvalidOperand)?;

    //1. read value from stack at esp
    let esp = cpu.registers.esp;
    let value = memory.read_u32(esp)?; //4 bytes

    //2. increment esp by 4
    //(check for overflow)
    if esp > 0xFFFF_FFFC {
        return Err(ExecutionError::StackOverflow);
    }
    cpu.registers.esp = esp.wrapping_add(4); //?

    //3.write to dest
    match dest_operand {
        Operand::Register(reg_name) => {
            cpu.registers.set(reg_name, value);
        }
        Operand::Memory(_addr_mode) => {
            //compute address from addr_mode
            //let addr = cpu.effective_address(&dest_operand, memory)?;
            //memory.write_u32(addr, value)?;
            return Err(ExecutionError::InvalidOperand);
        }
        _ => {
            return Err(ExecutionError::InvalidOperand);
        }
    }

    //4.advance instruction ptr
    cpu.registers.advance_ip(instruction.length as u32);

    Ok(())
}
