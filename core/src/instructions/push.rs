//! PUSH instruction
//! Pushes a value onto the stack.

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
            ExecutionError::InvalidOperand => write!(f, "Invalid operand for PUSH"),
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

/// Execute PUSH instruction
///
/// Pushes a value onto the stack:
/// 1. Decrement ESP by 4 (stack grows downwards)
/// 2. Read value from source register
/// 3. Write 4 bytes to [ESP]
/// 4. Advance EIP to next instruction
///
/// **Notes:**
/// - Flags not affected
/// - Assumes 32-bit operands
pub fn execute(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    // Get the source register value
    let src_value = match instruction.src {
        Some(Operand::Register(reg)) => cpu.registers.get(reg),
        Some(Operand::Immediate(val)) => val,
        _ => return Err(ExecutionError::InvalidOperand),
    };
///
    ///safetly check, prevent wraparound when decrementing ESP
    if cpu.registers.esp < 4 {
        return Err(ExecutionError::StackOverflow);
    }
    // Decrement ESP and write value to stack
    let new_esp = memory.push_u32(cpu.registers.esp, src_value)?;
///
    // Update ESP
    cpu.registers.esp = new_esp;
    
    // Advance EIP
    cpu.registers.advance_ip(instruction.length as u32);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::{Instruction, Opcode, Operand};
    use crate::cpu::RegisterName;
    
    #[test]
    fn test_push_eax() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);
        
        cpu.registers.eax = 0x12345678;
        cpu.registers.esp = 0x00FF0000;
        cpu.registers.eip = 0x1000;
        
        let instruction = Instruction {
            opcode: Opcode::PUSH,
            dest: None,
            src: Some(Operand::Register(RegisterName::EAX)),
            length: 1,
        };
        
        execute(&mut cpu, &mut memory, &instruction).unwrap();
        
        assert_eq!(cpu.registers.esp, 0x00FEFFFC);
        assert_eq!(memory.read_u32(0x00FEFFFC).unwrap(), 0x12345678);
        assert_eq!(cpu.registers.eip, 0x1001);
    }

    #[test]
    fn test_push_stack_overflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);

        cpu.registers.eax = 0x12345678;
        cpu.registers.esp = 0x00000003;
        cpu.registers.eip = 0x1000;

        let instruction = Instruction {
            opcode: Opcode::PUSH,
            dest: None,
            src: Some(Operand::Register(RegisterName::EAX)),
            length: 1,
        };

        let err = execute(&mut cpu, &mut memory, &instruction).unwrap_err();
        assert!(matches!(err, ExecutionError::StackOverflow));
        assert_eq!(cpu.registers.esp, 0x00000003);
        assert_eq!(cpu.registers.eip, 0x1000);
    }
}
