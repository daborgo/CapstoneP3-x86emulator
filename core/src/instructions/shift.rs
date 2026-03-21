use std::fmt;

use crate::cpu::CPU;
use crate::memory::Memory;
use crate::decoder::{Instruction, Operand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidOperand,
    MemoryError(crate::memory::MemoryError),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidOperand => write!(f, "Invalid operand for shift instruction"),
            ExecutionError::MemoryError(e) => write!(f, "Memory error: {}", e),
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<crate::memory::MemoryError> for ExecutionError {
    fn from(err: crate::memory::MemoryError) -> Self {
        ExecutionError::MemoryError(err)
    }
}

fn get_dst_and_count(
    cpu: &CPU,
    instruction: &Instruction,
) -> Result<(crate::cpu::RegisterName, u32), ExecutionError> {
    let dst_op = instruction.dest.ok_or(ExecutionError::InvalidOperand)?;
    let count_op = instruction.src.ok_or(ExecutionError::InvalidOperand)?;

    let dst_reg = match dst_op {
        Operand::Register(r) => r,
        _ => return Err(ExecutionError::InvalidOperand),
    };

    // x86 masks shift count to 5 bits (0–31)
    let count: u32 = match count_op {
        Operand::Immediate(v) => v & 0x1F,
        Operand::Register(r) => cpu.registers.get(r) & 0x1F, // CL form
        _ => return Err(ExecutionError::InvalidOperand),
    };

    Ok((dst_reg, count))
}

/// SHL r/m32, imm8 – logical/arithmetic shift left
pub fn execute_shl(cpu: &mut CPU, _memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    let (dst_reg, count) = get_dst_and_count(cpu, instruction)?;
    let val = cpu.registers.get(dst_reg);

    if count == 0 {
        cpu.registers.advance_ip(instruction.length as u32);
        return Ok(());
    }

    let result = val.wrapping_shl(count);
    cpu.registers.set(dst_reg, result);

    // CF = last bit shifted out (bit 32-count of original)
    cpu.flags.cf = if count <= 32 {
        ((val >> (32 - count)) & 1) != 0
    } else {
        false
    };
    // OF defined only for count==1: CF XOR MSB of result
    if count == 1 {
        cpu.flags.of = cpu.flags.cf ^ ((result >> 31) != 0);
    }
    cpu.flags.sf = (result >> 31) != 0;
    cpu.flags.zf = result == 0;
    let lowest_byte = (result & 0xFF) as u8;
    cpu.flags.pf = lowest_byte.count_ones() % 2 == 0;

    cpu.registers.advance_ip(instruction.length as u32);
    Ok(())
}

/// SHR r/m32, imm8 – logical shift right (unsigned)
pub fn execute_shr(cpu: &mut CPU, _memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    let (dst_reg, count) = get_dst_and_count(cpu, instruction)?;
    let val = cpu.registers.get(dst_reg);

    if count == 0 {
        cpu.registers.advance_ip(instruction.length as u32);
        return Ok(());
    }

    let result = val.wrapping_shr(count);
    cpu.registers.set(dst_reg, result);

    // CF = last bit shifted out = bit (count-1) of original
    cpu.flags.cf = if count <= 32 {
        ((val >> (count - 1)) & 1) != 0
    } else {
        false
    };
    // OF defined for count==1: = original MSB
    if count == 1 {
        cpu.flags.of = (val >> 31) != 0;
    }
    cpu.flags.sf = (result >> 31) != 0;
    cpu.flags.zf = result == 0;
    let lowest_byte = (result & 0xFF) as u8;
    cpu.flags.pf = lowest_byte.count_ones() % 2 == 0;

    cpu.registers.advance_ip(instruction.length as u32);
    Ok(())
}

/// SAR r/m32, imm8 – arithmetic shift right (signed)
pub fn execute_sar(cpu: &mut CPU, _memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    let (dst_reg, count) = get_dst_and_count(cpu, instruction)?;
    let val = cpu.registers.get(dst_reg) as i32;

    if count == 0 {
        cpu.registers.advance_ip(instruction.length as u32);
        return Ok(());
    }

    // Rust's >> on i32 is arithmetic shift
    let result = val.wrapping_shr(count) as u32;
    cpu.registers.set(dst_reg, result);

    // CF = last bit shifted out
    cpu.flags.cf = ((val >> (count - 1)) & 1) != 0;
    // OF = 0 for count==1 (SAR never produces signed overflow)
    if count == 1 {
        cpu.flags.of = false;
    }
    cpu.flags.sf = (result >> 31) != 0;
    cpu.flags.zf = result == 0;
    let lowest_byte = (result & 0xFF) as u8;
    cpu.flags.pf = lowest_byte.count_ones() % 2 == 0;

    cpu.registers.advance_ip(instruction.length as u32);
    Ok(())
}
