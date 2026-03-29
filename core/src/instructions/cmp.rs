//! CMP Instruction Implementation
//!
//! This module implements the CMP instruction which compares
//! two operands by subtracting them and setting the flags.

use std::fmt;
use crate::cpu::CPU;
use crate::memory::Memory;
use crate::decoder::{Instruction, Operand};

/// Execution errors for the CMP instruction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    /// Invalid operand types (e.g., CMP immediate, immediate)
    InvalidOperand,
    
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidOperand => {
                write!(f, "Invalid operand for instruction")
            },
        }
    }
}

impl std::error::Error for ExecutionError {}

pub fn execute(cpu: &mut CPU, _memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    
    let dest_operand = instruction.dest.ok_or(ExecutionError::InvalidOperand)?;
    let src_operand = instruction.src.ok_or(ExecutionError::InvalidOperand)?;

    let val_a = match dest_operand {
        Operand::Register(reg_name) => {
            cpu.registers.get(reg_name)
        },
        _ => {
            return Err(ExecutionError::InvalidOperand);
        }
    };

    let val_b = match src_operand {
        Operand::Register(reg_name) => {
            cpu.registers.get(reg_name)
        },
        Operand::Immediate(val) => {
            val
        },
        _ => {
            // Memory operands not supported yet
            return Err(ExecutionError::InvalidOperand);
        }
    };

    let result = val_a.wrapping_sub(val_b);

    cpu.flags.calculate_sub_flags(val_a, val_b, result);

    cpu.registers.advance_ip(instruction.length as u32);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::{Instruction, Opcode, Operand};
    use crate::cpu::{CPU, RegisterName};
    use crate::memory::Memory;

    // Helper to create a CMP instruction
    fn create_cmp_instr(dest: Operand, src: Operand) -> Instruction {
        Instruction {
            opcode: Opcode::CMP,
            dest: Some(dest),
            src: Some(src),
            length: 2, // Assume 2 bytes for simplicity in tests
        }
    }

    #[test]
    fn test_cmp_reg_reg_equal() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        cpu.registers.eax = 100;
        cpu.registers.ebx = 100;
        cpu.registers.eip = 0x1000;

        // CMP EAX, EBX
        let instr = create_cmp_instr(
            Operand::Register(RegisterName::EAX),
            Operand::Register(RegisterName::EBX)
        );

        execute(&mut cpu, &mut memory, &instr).unwrap();

        // 100 - 100 = 0
        // Verify Flags: Zero Flag (ZF) should be set
        assert!(cpu.flags.zf);
        // Verify: Not negative (SF), No carry (CF), No overflow (OF)
        assert!(!cpu.flags.sf);
        assert!(!cpu.flags.cf);
        assert!(!cpu.flags.of);

        // Verify registers are unchanged
        assert_eq!(cpu.registers.eax, 100);
        assert_eq!(cpu.registers.ebx, 100);
        // Verify EIP advanced
        assert_eq!(cpu.registers.eip, 0x1002);
    }

    #[test]
    fn test_cmp_reg_reg_less_than() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        cpu.registers.eax = 5;
        cpu.registers.ebx = 10;
        
        // CMP EAX, EBX
        let instr = create_cmp_instr(
            Operand::Register(RegisterName::EAX),
            Operand::Register(RegisterName::EBX)
        );

        execute(&mut cpu, &mut memory, &instr).unwrap();

        // 5 - 10 = -5
        // Verify Flags:
        assert!(!cpu.flags.zf); // Not zero
        assert!(cpu.flags.sf);  // Sign Flag (negative result)
        assert!(cpu.flags.cf);  // Carry Flag (borrow occurred)
        assert!(!cpu.flags.of); // No signed overflow
    }

    #[test]
    fn test_cmp_reg_reg_greater_than() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        cpu.registers.eax = 20;
        cpu.registers.ebx = 10;
        
        // CMP EAX, EBX
        let instr = create_cmp_instr(
            Operand::Register(RegisterName::EAX),
            Operand::Register(RegisterName::EBX)
        );

        execute(&mut cpu, &mut memory, &instr).unwrap();

        // 20 - 10 = 10
        // Verify Flags:
        assert!(!cpu.flags.zf); // Not zero
        assert!(!cpu.flags.sf); // Not negative
        assert!(!cpu.flags.cf); // No borrow
        assert!(!cpu.flags.of); // No overflow
    }

    #[test]
    fn test_cmp_reg_imm_equal() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        cpu.registers.eax = 50;
        
        // CMP EAX, 50
        let instr = create_cmp_instr(
            Operand::Register(RegisterName::EAX),
            Operand::Immediate(50)
        );

        execute(&mut cpu, &mut memory, &instr).unwrap();

        // 50 - 50 = 0
        // Verify Flags: Zero Flag (ZF) should be set
        assert!(cpu.flags.zf);
        assert!(!cpu.flags.sf);
        assert!(!cpu.flags.cf);
    }

    #[test]
    fn test_cmp_signed_overflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        
        // Test signed overflow: 0x7FFFFFFF - (-1)
        cpu.registers.eax = 0x7FFFFFFF; // Max positive signed
        cpu.registers.ebx = 0xFFFFFFFF; // -1
        
        // CMP EAX, EBX
        let instr = create_cmp_instr(
            Operand::Register(RegisterName::EAX),
            Operand::Register(RegisterName::EBX)
        );

        execute(&mut cpu, &mut memory, &instr).unwrap();
        
        assert!(cpu.flags.of);  // Overflow Flag
        assert!(cpu.flags.sf);  // Sign Flag
        assert!(!cpu.flags.zf); // Not zero
        assert!(cpu.flags.cf); // Unsigned borrow: 0x7FFFFFFF < 0xFFFFFFFF
    }
}

