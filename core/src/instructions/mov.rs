// MOV Instruction implementation
// This module implements the MOV instruction which copies data from a source operand to a destination operand


<<<<<<< Updated upstream
use std::fmt

use crate::cpu::{CPU, Operand};
use crate::memory::Memoryuse 
use crate::decoder::{Instruction, Operand}

pub fn execute(cpu: &mut CPU, dest: Operand, src: Operand) {
=======
use std::fmt;

use crate::cpu::{CPU, Operand};
use crate::memory::Memoryuse;
use crate::decoder::{Instruction, Operand};

pub enum ExecutionError {
    InvalidOperand,

    MemoryError(crate::memoery::MemoryError),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidOperand => write!(f, "Invalid operand for MOV instruction"),
            ExecutionError::MemoryError(err) => write!(f, "Memory error: {}", err),
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<crate::memory::MemoryError> for ExecutionError {
    fn from(err: crate::memory::MemoryError) -> Self {
        ExecutionError::MemoryError(err)
    }
}


pub fn execute(cpu: &mut CPU, dest: Operand, src: Operand) -> Result<(), ExecutionError> {
>>>>>>> Stashed changes
    // MOV requires both a source and destination operand
    let src_operand = instruction.src.ok_or(ExecutionError::InvalidOperand)?;
    let dest_operand = instruction.dest.ok_or(ExecutionError::InvalidOperand)?;

    // Retrieve the value from the source operand
    let value = match src_operand {
        Operand::Register(reg_name) => cpu.registers.get(reg_name),
        Operand::Immediate(val) => val,
        Operand::Memory(addr) => memory.read_u32(addr)?,
    };

   match dest_operand {
        Operand::Register(reg_name) => {
            cpu.registers.set(reg_name, value);
        }
        Operand::Memory(addr) => {
            memory.write_u32(addr, value)?;
        }
        Operand::Immediate(_) => {
            // Cannot write into an immediate value
            return Err(ExecutionError::InvalidOperand);
        }
    }
<<<<<<< Updated upstream
=======
}

#{cfg(test)}
mod tests {
    use super::*;
    use crate::decoder::{Instruction, Opcode}
>>>>>>> Stashed changes
}