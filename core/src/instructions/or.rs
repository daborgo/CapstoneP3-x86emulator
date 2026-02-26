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
            ExecutionError::InvalidOperand => write!(f, "Invalid operand for OR"),
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

/// OR r/m32, r32 or OR r/m32, imm32
pub fn execute(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    let dst_op = instruction.dest.ok_or(ExecutionError::InvalidOperand)?;
    let src_op = instruction.src.ok_or(ExecutionError::InvalidOperand)?;

    let dst_reg = match dst_op {
        Operand::Register(r) => r,
        _ => return Err(ExecutionError::InvalidOperand),
    };

    let src_val: u32 = match src_op {
        Operand::Register(r) => cpu.registers.get(r),
        Operand::Immediate(v) => v,
        Operand::Memory(addr) => memory.read_u32(addr)?,
    };

    let result = cpu.registers.get(dst_reg) | src_val;
    cpu.registers.set(dst_reg, result);

    // OR: CF=0, OF=0, ZF, SF, PF updated
    cpu.flags.cf = false;
    cpu.flags.of = false;
    cpu.flags.af = false;
    cpu.flags.zf = result == 0;
    cpu.flags.sf = (result & 0x8000_0000) != 0;
    let lowest_byte = (result & 0xFF) as u8;
    cpu.flags.pf = lowest_byte.count_ones() % 2 == 0;

    cpu.registers.advance_ip(instruction.length as u32);
    Ok(())
}
