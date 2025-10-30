//! PUSH Instruction Implementation
//! 
//! This module implements the PUSH instruction which pushes a register
//! value onto the stack.

use std::fmt;

use crate::cpu::CPU;
use crate::memory::Memory;
use crate::decoder::{Instruction, Operand};

/// Execution errors for instructions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    /// Invalid operand type for this instruction
    InvalidOperand,
    
    /// Memory access error during execution
    MemoryError(crate::memory::MemoryError),
    
    /// Stack overflow (ESP would wrap around)
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

/// Execute a PUSH instruction
/// 
/// PUSH instruction pushes a register value onto the stack.
/// 
/// Operation:
/// 1. Get the value from the source register
/// 2. Decrement ESP by 4 (stack grows downward)
/// 3. Write the value to memory at [ESP]
/// 4. Advance EIP to next instruction
/// 
/// # Arguments
/// * `cpu` - Mutable reference to CPU state
/// * `memory` - Mutable reference to memory system
/// * `instruction` - The decoded PUSH instruction
/// 
/// # Returns
/// * `Ok(())` - Instruction executed successfully
/// * `Err(ExecutionError)` - If execution fails
/// 
/// # Example
/// ```rust
/// use web_x86_core::cpu::CPU;
/// use web_x86_core::memory::Memory;
/// use web_x86_core::decoder::{Instruction, Opcode, Operand};
/// use web_x86_core::cpu::RegisterName;
/// use web_x86_core::instructions::push;
/// 
/// let mut cpu = CPU::new();
/// let mut memory = Memory::new(0x1000000);
/// 
/// // Set up test: EAX = 0x12345678, ESP = 0x00FF0000
/// cpu.registers.eax = 0x12345678;
/// cpu.registers.esp = 0x00FF0000;
/// 
/// // Create PUSH EAX instruction
/// let instruction = Instruction {
///     opcode: Opcode::PUSH,
///     dest: None,
///     src: Some(Operand::Register(RegisterName::EAX)),
///     length: 1,
/// };
/// 
/// // Execute PUSH EAX
/// push::execute(&mut cpu, &mut memory, &instruction).unwrap();
/// 
/// // Verify: ESP decremented, value pushed to stack
/// assert_eq!(cpu.registers.esp, 0x00FEFFFC);
/// assert_eq!(memory.read_u32(0x00FEFFFC).unwrap(), 0x12345678);
/// ```
pub fn execute(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    // PUSH instruction should have a source operand (the register to push)
    let src_operand = instruction.src.ok_or(ExecutionError::InvalidOperand)?;
    
    // Get the value to push
    let value_to_push = match src_operand {
        Operand::Register(reg_name) => {
            // Get value from the specified register
            cpu.registers.get(reg_name)
        },
        Operand::Immediate(val) => {
            // PUSH immediate values (not implemented yet, but structure is here)
            val
        },
        Operand::Memory(_) => {
            // PUSH memory values (not implemented yet)
            return Err(ExecutionError::InvalidOperand);
        },
    };
    
    // Check for stack overflow before pushing
    // If ESP is at 0x00000000, decrementing by 4 would wrap to 0xFFFFFFFC
    // This is technically valid in x86, but we'll treat it as overflow for safety
    if cpu.registers.esp < 4 {
        return Err(ExecutionError::StackOverflow);
    }
    
    // Push the value onto the stack
    // This decrements ESP and writes the value to memory
    let new_esp = memory.push_u32(cpu.registers.esp, value_to_push)?;
    
    // Update ESP with the new stack pointer
    cpu.registers.esp = new_esp;
    
    // Advance instruction pointer to next instruction
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
        let mut memory = Memory::new(0x1000000);  // 16MB memory
        
        // Set up test state with smaller addresses
        cpu.registers.eax = 0x12345678;
        cpu.registers.esp = 0x00FF0000;  // Use smaller stack address
        cpu.registers.eip = 0x1000;
        
        // Create PUSH EAX instruction
        let instruction = Instruction {
            opcode: Opcode::PUSH,
            dest: None,
            src: Some(Operand::Register(RegisterName::EAX)),
            length: 1,
        };
        
        // Execute instruction
        execute(&mut cpu, &mut memory, &instruction).unwrap();
        
        // Verify results
        assert_eq!(cpu.registers.esp, 0x00FEFFFC);  // ESP decremented by 4
        assert_eq!(cpu.registers.eip, 0x1001);      // EIP advanced by 1
        assert_eq!(memory.read_u32(0x00FEFFFC).unwrap(), 0x12345678);  // Value pushed
    }
    
    #[test]
    fn test_push_ebx() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);  // 16MB memory
        
        // Set up test state with smaller addresses
        cpu.registers.ebx = 0xDEADBEEF;
        cpu.registers.esp = 0x00800000;  // Use smaller stack address
        
        // Create PUSH EBX instruction
        let instruction = Instruction {
            opcode: Opcode::PUSH,
            dest: None,
            src: Some(Operand::Register(RegisterName::EBX)),
            length: 1,
        };
        
        // Execute instruction
        execute(&mut cpu, &mut memory, &instruction).unwrap();
        
        // Verify results
        assert_eq!(cpu.registers.esp, 0x007FFFFC);  // ESP decremented by 4
        assert_eq!(memory.read_u32(0x007FFFFC).unwrap(), 0xDEADBEEF);  // Value pushed
    }
    
    #[test]
    fn test_push_multiple() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000000);  // 16MB memory
        
        // Set up test state with smaller addresses
        cpu.registers.eax = 0x11111111;
        cpu.registers.ebx = 0x22222222;
        cpu.registers.esp = 0x00FF0000;  // Use smaller stack address
        
        // Push EAX
        let push_eax = Instruction {
            opcode: Opcode::PUSH,
            dest: None,
            src: Some(Operand::Register(RegisterName::EAX)),
            length: 1,
        };
        execute(&mut cpu, &mut memory, &push_eax).unwrap();
        
        // Push EBX
        let push_ebx = Instruction {
            opcode: Opcode::PUSH,
            dest: None,
            src: Some(Operand::Register(RegisterName::EBX)),
            length: 1,
        };
        execute(&mut cpu, &mut memory, &push_ebx).unwrap();
        
        // Verify stack contents (EBX pushed first, then EAX)
        assert_eq!(cpu.registers.esp, 0x00FEFFF8);  // ESP decremented by 8 total
        assert_eq!(memory.read_u32(0x00FEFFFC).unwrap(), 0x11111111);  // EAX at higher address
        assert_eq!(memory.read_u32(0x00FEFFF8).unwrap(), 0x22222222);  // EBX at lower address
    }
    
    #[test]
    fn test_push_stack_overflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::default();
        
        // Set ESP to a value that would overflow when decremented
        cpu.registers.esp = 2;  // Less than 4, so decrementing by 4 would wrap
        
        let instruction = Instruction {
            opcode: Opcode::PUSH,
            dest: None,
            src: Some(Operand::Register(RegisterName::EAX)),
            length: 1,
        };
        
        // Should fail with stack overflow
        assert!(execute(&mut cpu, &mut memory, &instruction).is_err());
    }
    
    #[test]
    fn test_push_no_source_operand() {
        let mut cpu = CPU::new();
        let mut memory = Memory::default();
        
        // Create PUSH instruction without source operand
        let instruction = Instruction {
            opcode: Opcode::PUSH,
            dest: None,
            src: None,  // No source operand!
            length: 1,
        };
        
        // Should fail with invalid operand
        assert!(execute(&mut cpu, &mut memory, &instruction).is_err());
    }
}

