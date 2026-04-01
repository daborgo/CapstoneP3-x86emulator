use std::fmt;

use crate::cpu::CPU;
use crate::memory::Memory;
use crate::decoder::{Instruction, Operand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidOperand,
    DivisionByZero,
    IntegerOverflow,
    MemoryError(crate::memory::MemoryError),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidOperand => write!(f, "Invalid operand for MUL/IDIV/CDQ"),
            ExecutionError::DivisionByZero => write!(f, "Division by zero"),
            ExecutionError::IntegerOverflow => write!(f, "Integer overflow in division result"),
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

/// MUL r/m32: unsigned multiply EDX:EAX = EAX * src
pub fn execute_mul(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    let src_op = instruction.src.ok_or(ExecutionError::InvalidOperand)?;
    let src: u32 = match src_op {
        Operand::Register(r) => cpu.registers.get(r),
        Operand::Memory(addr) => memory.read_u32(addr)?,
        Operand::Immediate(_) => return Err(ExecutionError::InvalidOperand),
    };

    let eax = cpu.registers.eax;
    let result: u64 = (eax as u64).wrapping_mul(src as u64);

    cpu.registers.eax = (result & 0xFFFF_FFFF) as u32;
    cpu.registers.edx = ((result >> 32) & 0xFFFF_FFFF) as u32;

    // CF and OF set if high half is non-zero
    let high_nonzero = cpu.registers.edx != 0;
    cpu.flags.cf = high_nonzero;
    cpu.flags.of = high_nonzero;

    cpu.registers.advance_ip(instruction.length as u32);
    Ok(())
}

/// IDIV r/m32: signed divide EDX:EAX / src
/// EAX = quotient (truncated toward zero), EDX = remainder
pub fn execute_idiv(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    let src_op = instruction.src.ok_or(ExecutionError::InvalidOperand)?;
    let divisor: i64 = match src_op {
        Operand::Register(r) => cpu.registers.get(r) as i32 as i64,
        Operand::Memory(addr) => memory.read_u32(addr)? as i32 as i64,
        Operand::Immediate(_) => return Err(ExecutionError::InvalidOperand),
    };

    if divisor == 0 {
        return Err(ExecutionError::DivisionByZero);
    }

    // Build 64-bit signed dividend from EDX:EAX
    let eax = cpu.registers.eax as u64;
    let edx = cpu.registers.edx as u64;
    let dividend_u64: u64 = (edx << 32) | eax;
    let dividend: i64 = dividend_u64 as i64;

    let quotient = dividend / divisor;
    let remainder = dividend % divisor;

    // Check that quotient fits in i32
    if quotient > i32::MAX as i64 || quotient < i32::MIN as i64 {
        return Err(ExecutionError::IntegerOverflow);
    }

    cpu.registers.eax = quotient as i32 as u32;
    cpu.registers.edx = remainder as i32 as u32;

    // Flags are undefined after IDIV; we leave them unchanged
    cpu.registers.advance_ip(instruction.length as u32);
    Ok(())
}

/// IMUL r32, r/m32: signed multiply dest = dest * src (32-bit result)
pub fn execute_imul(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    let dst_reg = match instruction.dest {
        Some(Operand::Register(r)) => r,
        _ => return Err(ExecutionError::InvalidOperand),
    };
    let src_op = instruction.src.ok_or(ExecutionError::InvalidOperand)?;
    let src: u32 = match src_op {
        Operand::Register(r) => cpu.registers.get(r),
        Operand::Memory(addr) => memory.read_u32(addr)?,
        Operand::Immediate(_) => return Err(ExecutionError::InvalidOperand),
    };

    let dst_val = cpu.registers.get(dst_reg) as i32;
    let src_val = src as i32;
    let result64 = (dst_val as i64).wrapping_mul(src_val as i64);
    let result32 = result64 as i32 as u32;

    cpu.registers.set(dst_reg, result32);

    // CF and OF set if sign-extended 32-bit result != full 64-bit result
    let overflow = result64 != (result32 as i32 as i64);
    cpu.flags.cf = overflow;
    cpu.flags.of = overflow;

    cpu.registers.advance_ip(instruction.length as u32);
    Ok(())
}

/// CDQ: sign-extend EAX into EDX:EAX (convert doubleword to quadword)
pub fn execute_cdq(cpu: &mut CPU, _memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    let sign_bit = (cpu.registers.eax & 0x8000_0000) != 0;
    cpu.registers.edx = if sign_bit { 0xFFFF_FFFF } else { 0 };
    cpu.registers.advance_ip(instruction.length as u32);
    Ok(())
}
