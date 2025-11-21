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
//2. Increment ESP by 4 (stack grows downwards)
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
//`cpu` - Mut reference to CPU state
//`memory` - Mut reference to memory system
//`instruction` -  decoded POP instruction

pub fn execute(
    cpu: &mut CPU,
    memory: &mut Memory,
    instruction: &Instruction,
) -> Result<(), ExecutionError> {
    //destination operand
    let dest_operand = instruction.dest.ok_or(ExecutionError::InvalidOperand)?;

    //1. read value from stack
    let esp = cpu.registers.esp;

    //(check for overflow)
    if esp > 0xFFFF_FFFC {
        return Err(ExecutionError::StackOverflow);
    }

    let value = memory.read_u32(esp)?; //4 bytes

    //3.write to dest
    match dest_operand {
        Operand::Register(reg_name) => {
            cpu.registers.set(reg_name, value);
        }
        Operand::Memory(addr) => {
            //direct memory addresing mode (for now)
            memory.write_u32(addr, value)?;
        }
        Operand::Immediate(_) => {
            //n/a dont pop into an immediate
            return Err(ExecutionError::InvalidOperand);
        }
    }

    cpu.registers.esp = esp.wrapping_add(4); //?

    //4.advance instruction ptr
    cpu.registers.advance_ip(instruction.length as u32);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::{RegisterName, Registers};
    use crate::decoder::{Instruction, Opcode, Operand};
    use crate::memory;

    #[test]
    fn test_pop() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);

        //set up dummy regs
        cpu.registers.esp = 0x2000; //small stack
        cpu.registers.eip = 0x1000;

        //setting [esp] to some random memory location
        memory.write_u32(0x2000, 0xFFFF);

        //set up instruction
        let instruction = Instruction {
            opcode: Opcode::POP,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: None,
            length: 1,
        };

        //execute instruction
        execute(&mut cpu, &mut memory, &instruction).unwrap();

        //verify results
        //esp should have incremented by 4
        //eip should have also incremented
        assert_eq!(cpu.registers.esp, 0x2000 + 4);
        assert_eq!(cpu.registers.eax, 0xFFFF);
        assert_eq!(cpu.registers.eip, 0x1000 + 1);
    }

    #[test]
    fn test_pop_invalid_operand() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);

        cpu.registers.esp = 0x2000;
        cpu.registers.eip = 0x1000;
        memory.write_u32(0x2000, 0xFFFF);

        let instruction = Instruction {
            opcode: Opcode::POP,
            dest: Some(Operand::Immediate(10)),
            src: None,
            length: 1,
        };

        let err = execute(&mut cpu, &mut memory, &instruction).unwrap_err();
        assert!(matches!(err, ExecutionError::InvalidOperand));

        //ensure that registers are unchanged
        assert_eq!(cpu.registers.esp, 0x2000);
        assert_eq!(cpu.registers.eip, 0x1000);
    }

    #[test]
    fn test_pop_missing_dest() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);

        cpu.registers.esp = 0x2000;
        cpu.registers.eip = 0x1000;
        memory.write_u32(0x2000, 0xFFFF);

        let instruction = Instruction {
            opcode: Opcode::POP,
            dest: None,
            src: None,
            length: 1,
        };

        let err = execute(&mut cpu, &mut memory, &instruction).unwrap_err();
        assert!(matches!(err, ExecutionError::InvalidOperand));
        assert_eq!(cpu.registers.esp, 0x2000);
        assert_eq!(cpu.registers.eip, 0x1000);
    }

    #[test]
    fn test_pop_stack_overflow_error() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x100000); //1 mb

        cpu.registers.esp = 0xFFFF_FFFF;
        cpu.registers.eip = 0x1000;
        memory.write_u32(0x2000, 0x1);

        let instruction = Instruction {
            opcode: Opcode::POP,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: None,
            length: 1,
        };

        let err = execute(&mut cpu, &mut memory, &instruction).unwrap_err();
        assert!(matches!(err, ExecutionError::StackOverflow));
        assert_eq!(cpu.registers.esp, 0xFFFF_FFFF);
        assert_eq!(cpu.registers.eip, 0x1000);
    }
}