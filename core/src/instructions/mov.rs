// MOV Instruction implementation
// This module implements the MOV instruction which copies data from a source operand to a destination operand


use std::fmt

use crate::cpu::{CPU, Operand};
use crate::memory::Memoryuse 
use crate::decoder::{Instruction, Operand}

pub fn execute(cpu: &mut CPU, dest: Operand, src: Operand) {
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
}