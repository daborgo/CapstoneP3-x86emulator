use std::fmt;

// added from core
use crate::cpu::CPU;
use crate::memory::Memory;
use crate::decoder::{Instruction, Operand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidOperand,
    MemoryError(crate::memory::MemoryError),
    StackOverflow,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidOperand => {
                write!(f, "Invalid operand for instruction")
            },
            ExecutionError::MemoryError(err) => {
                write!(f, "Memory error: {}", err)
            },
            ExecutionError::StackOverflow => {
                write!(f, "Stack overflow")
            },
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<crate::memory::MemoryError> for ExecutionError {
    fn from(err: crate::memory::MemoryError) -> Self {
        ExecutionError::MemoryError(err)
    }
}

pub fn add(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    let load_operand = |op: Operand| -> Result<u32, ExecutionError> {
        match op {
            Operand::Register(r) => Ok(cpu.registers.get(r)),
            Operand::Immediate(v) => Ok(v),
            Operand::Memory(addr) => {
                Ok(memory.read_u32(addr)?)
            }
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
    let sum = a.wrapping_add(b);

    cpu.registers.set(dst_reg, sum);
    cpu.registers.advance_ip(instruction.length as u32);
    cpu.flags.calculate_add_flags(a, b, sum);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Instruction;
    use crate::decoder::Operand;
    use crate::decoder::Opcode;
    use crate::cpu::RegisterName;
    use crate::memory::Memory as Ram;

    #[test]
    fn test_add_registers() {
        let mut cpu = crate::cpu::CPU::new();
        let mut memory = Ram::new(0x1000000);

        cpu.registers.eax = 5;
        cpu.registers.ebx = 3;
        cpu.registers.eip = 0x1000;

        let instr = Instruction {
            opcode: Opcode::PUSH, // opcode value is unused by executor here; keep consistent type
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Register(RegisterName::EBX)),
            length: 1,
        };

        // Execute ADD EAX, EBX
        add(&mut cpu, &mut memory, &instr).unwrap();

        assert_eq!(cpu.registers.eax, 8);
        assert_eq!(cpu.registers.eip, 0x1001);
        assert!(!cpu.flags.cf);
        assert!(!cpu.flags.zf);
    }

    #[test]
    fn test_add_immediate_and_flags() {
        let mut cpu = crate::cpu::CPU::new();
        let mut memory = Ram::new(0x1000000);

        cpu.registers.eax = 0xFFFFFFFF;
        cpu.registers.eip = 0x1000;

        let instr = Instruction {
            opcode: Opcode::PUSH,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Immediate(1)),
            length: 1,
        };

        // Execute ADD EAX, 1 -> wraps to 0, CF should be set, ZF set
        add(&mut cpu, &mut memory, &instr).unwrap();

        assert_eq!(cpu.registers.eax, 0);
        assert_eq!(cpu.registers.eip, 0x1001);
        assert!(cpu.flags.cf);
        assert!(cpu.flags.zf);
    }

    #[test]
    fn test_add_invalid_destination() {
        let mut cpu = crate::cpu::CPU::new();
        let mut memory = Ram::new(0x1000000);

        cpu.registers.ebx = 1;
        cpu.registers.eip = 0x1000;

        let instr = Instruction {
            opcode: Opcode::PUSH,
            dest: Some(Operand::Immediate(1)), // invalid destination
            src: Some(Operand::Register(RegisterName::EBX)),
            length: 1,
        };

        let res = add(&mut cpu, &mut memory, &instr);
        assert_eq!(res.unwrap_err(), super::ExecutionError::InvalidOperand);
    }

    #[test]
    fn test_add_signed_overflow_positive_to_negative() {
        let mut cpu = crate::cpu::CPU::new();
        let mut memory = Ram::new(0x1000000);

        // INT_MAX + 1 => 0x7FFFFFFF + 1 = 0x80000000 (negative in signed), OF should be set
        cpu.registers.eax = 0x7FFFFFFF;
        cpu.registers.eip = 0x4000;

        let instr = Instruction {
            opcode: Opcode::PUSH,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Immediate(1)),
            length: 1,
        };

        add(&mut cpu, &mut memory, &instr).unwrap();

        assert_eq!(cpu.registers.eax, 0x80000000);
        assert_eq!(cpu.registers.eip, 0x4001);
        assert!(!cpu.flags.cf); // no unsigned carry out
        assert!(!cpu.flags.zf);
        // OF should be set because signed overflow occurred
        assert!(cpu.flags.of);
        // sign flag should reflect negative result
        assert!(cpu.flags.sf);
    }

    #[test]
    fn test_add_signed_overflow_negative_to_zero_with_carry() {
        let mut cpu = crate::cpu::CPU::new();
        let mut memory = Ram::new(0x1000000);

        // 0x80000000 + 0x80000000 = 0x00000000, CF=1, ZF=1, OF=1 (signed overflow)
        cpu.registers.eax = 0x80000000;
        cpu.registers.eip = 0x5000;

        let instr = Instruction {
            opcode: Opcode::PUSH,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Immediate(0x80000000)),
            length: 1,
        };

        add(&mut cpu, &mut memory, &instr).unwrap();

        assert_eq!(cpu.registers.eax, 0x00000000);
        assert_eq!(cpu.registers.eip, 0x5001);
        assert!(cpu.flags.cf);
        assert!(cpu.flags.zf);
        assert!(cpu.flags.of);
    }
}