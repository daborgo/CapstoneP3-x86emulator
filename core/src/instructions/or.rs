//! OR Instruction Implementation
//!
//! Implements bitwise OR for register destinations.

use std::fmt;

use crate::cpu::CPU;
use crate::decoder::{Instruction, Operand};
use crate::memory::Memory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidOperand,
    MemoryError(crate::memory::MemoryError),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidOperand => write!(f, "Invalid operand for OR instruction"),
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

pub fn execute(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    let load_operand = |op: Operand| -> Result<u32, ExecutionError> {
        match op {
            Operand::Register(r) => Ok(cpu.registers.get(r)),
            Operand::Immediate(v) => Ok(v),
            Operand::Memory(addr) => Ok(memory.read_u32(addr)?),
        }
    };

    let dst_op = instruction.dest.ok_or(ExecutionError::InvalidOperand)?;
    let src_op = instruction.src.ok_or(ExecutionError::InvalidOperand)?;

    let dst_reg = match dst_op {
        Operand::Register(r) => r,
        _ => return Err(ExecutionError::InvalidOperand),
    };

    let a = cpu.registers.get(dst_reg);
    let b = load_operand(src_op)?;
    let result = a | b;

    cpu.registers.set(dst_reg, result);
    cpu.flags.calculate_or_flags(result);
    cpu.registers.advance_ip(instruction.length as u32);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::RegisterName;
    use crate::decoder::{Instruction, Opcode, Operand};
    use crate::memory::Memory as Ram;

    #[test]
    fn test_or_registers() {
        let mut cpu = crate::cpu::CPU::new();
        let mut memory = Ram::new(0x1000000);

        cpu.registers.eax = 0xF0F0_0000;
        cpu.registers.ebx = 0x0000_0FF0;
        cpu.registers.eip = 0x1000;

        let instr = Instruction {
            opcode: Opcode::OR,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Register(RegisterName::EBX)),
            length: 2,
        };

        execute(&mut cpu, &mut memory, &instr).unwrap();

        assert_eq!(cpu.registers.eax, 0xF0F0_0FF0);
        assert_eq!(cpu.registers.eip, 0x1002);
        assert!(!cpu.flags.cf);
        assert!(!cpu.flags.of);
        assert!(!cpu.flags.zf);
    }

    #[test]
    fn test_or_immediate_sets_zero_flag() {
        let mut cpu = crate::cpu::CPU::new();
        let mut memory = Ram::new(0x1000000);

        cpu.registers.eax = 0x0000_0000;
        cpu.registers.eip = 0x1000;

        let instr = Instruction {
            opcode: Opcode::OR,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Immediate(0x0)),
            length: 6,
        };

        execute(&mut cpu, &mut memory, &instr).unwrap();

        assert_eq!(cpu.registers.eax, 0x0000_0000);
        assert_eq!(cpu.registers.eip, 0x1006);
        assert!(cpu.flags.zf);
        assert!(!cpu.flags.cf);
        assert!(!cpu.flags.of);
    }

    #[test]
    fn test_or_invalid_destination() {
        let mut cpu = crate::cpu::CPU::new();
        let mut memory = Ram::new(0x1000000);

        let instr = Instruction {
            opcode: Opcode::OR,
            dest: Some(Operand::Immediate(1)),
            src: Some(Operand::Register(RegisterName::EBX)),
            length: 2,
        };

        let res = execute(&mut cpu, &mut memory, &instr);
        assert_eq!(res.unwrap_err(), ExecutionError::InvalidOperand);
    }
}
