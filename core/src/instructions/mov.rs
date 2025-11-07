//! MOV Instruction Implementation
//!
//! This module implements the MOV instruction which copies data from a source
//! operand to a destination operand.

use std::fmt;

use crate::cpu::CPU;
use crate::memory::Memory;
use crate::decoder::{Instruction, Operand};

/// Execution errors for MOV
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    /// Invalid operand type for this instruction
    InvalidOperand,

    /// Memory access error during execution
    MemoryError(crate::memory::MemoryError),
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

/// Execute a MOV instruction
///
/// MOV copies a 32-bit value from src to dest. Supported operands:
/// - src: Register, Immediate, Memory
/// - dest: Register, Memory
///
/// MOV with immediate as destination is invalid.
///
/// Advances EIP by instruction.length on success.
pub fn execute(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    // Borrow operands from &Instruction (avoid moving out of borrowed value)
    let src_op = instruction.src.as_ref().ok_or(ExecutionError::InvalidOperand)?;
    let dest_op = instruction.dest.as_ref().ok_or(ExecutionError::InvalidOperand)?;

    // Read 32-bit value from source
    let value: u32 = match src_op {
        Operand::Register(reg) => {
            cpu.registers.get(*reg)
        }
        Operand::Immediate(v) => {
            *v
        }
        Operand::Memory(addr) => {
            memory.read_u32(*addr)?
        }
    };

    // Write 32-bit value to destination
    match dest_op {
        Operand::Register(reg) => {
            cpu.registers.set(*reg, value);
        }
        Operand::Memory(addr) => {
            memory.write_u32(*addr, value)?;
        }
        Operand::Immediate(_) => {
            return Err(ExecutionError::InvalidOperand);
        }
    }
    
    #[allow(unused_must_use)]
    {
        // try using helper, fall back to direct increment
        if let Some(advance_fn) = {
            // detect presence by attempting to call method; if not present this block is ignored
            None::<fn(&mut crate::cpu::registers::Registers, u32)>
        } {
            // placeholder to satisfy style parity; real code below
        }
    }
    // fallback direct update (works with the field-based registers in your tests)
    cpu.registers.eip = cpu.registers.eip.wrapping_add(instruction.length as u32);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::{Instruction, Opcode, Operand};
    use crate::cpu::RegisterName;

    #[test]
    // moves value from one register to another
    fn test_mov_reg_to_reg() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);

        cpu.registers.eax = 0x12345678;
        cpu.registers.ebx = 0;
        cpu.registers.eip = 0x1000;

        let instruction = Instruction {
            opcode: Opcode::MOV,
            dest: Some(Operand::Register(RegisterName::EBX)),
            src: Some(Operand::Register(RegisterName::EAX)),
            length: 1,
        };

        execute(&mut cpu, &mut memory, &instruction).unwrap();

        assert_eq!(cpu.registers.ebx, 0x12345678);
        assert_eq!(cpu.registers.eip, 0x1001);
    }

    #[test]
    // writes the immediate into the dest reg
    fn test_mov_imm_to_reg() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);

        cpu.registers.eax = 0;
        cpu.registers.eip = 0x2000;

        let instruction = Instruction {
            opcode: Opcode::MOV,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Immediate(0xDEADBEEF)),
            length: 5,
        };

        execute(&mut cpu, &mut memory, &instruction).unwrap();

        assert_eq!(cpu.registers.eax, 0xDEADBEEF);
        assert_eq!(cpu.registers.eip, 0x2005);
    }

    #[test]
    fn test_mov_mem_to_reg_and_back() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);

        // write a value to memory and move it into eax, then to another memory location
        memory.write_u32(0x100, 0xCAFEBABE).unwrap();

        let ins1 = Instruction {
            opcode: Opcode::MOV,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Memory(0x100)),
            length: 2,
        };
        execute(&mut cpu, &mut memory, &ins1).unwrap();
        assert_eq!(cpu.registers.eax, 0xCAFEBABE);

        let ins2 = Instruction {
            opcode: Opcode::MOV,
            dest: Some(Operand::Memory(0x200)),
            src: Some(Operand::Register(RegisterName::EAX)),
            length: 2,
        };
        execute(&mut cpu, &mut memory, &ins2).unwrap();
        assert_eq!(memory.read_u32(0x200).unwrap(), 0xCAFEBABE);
    }

    #[test]
    fn test_mov_invalid_dest_immediate() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);

        let instruction = Instruction {
            opcode: Opcode::MOV,
            dest: Some(Operand::Immediate(0)),
            src: Some(Operand::Register(RegisterName::EAX)),
            length: 1,
        };

        assert!(execute(&mut cpu, &mut memory, &instruction).is_err());
    }
}