//! Instructions Module
//!
//! This module contains all instruction implementations and the
//! instruction execution dispatcher.

use std::fmt;

pub mod pop;

use crate::cpu::CPU;
use crate::decoder::{Instruction, Opcode};
use crate::memory::Memory;

/// Instruction execution errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstructionError {
    /// Unsupported instruction
    UnsupportedInstruction(Opcode),

    /// Execution error from specific instruction
    ExecutionError(pop::ExecutionError),
}

impl fmt::Display for InstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionError::UnsupportedInstruction(opcode) => {
                write!(f, "Unsupported instruction: {}", opcode)
            }
            InstructionError::ExecutionError(err) => {
                write!(f, "Execution error: {}", err)
            }
        }
    }
}

impl std::error::Error for InstructionError {}

impl From<pop::ExecutionError> for InstructionError {
    fn from(err: pop::ExecutionError) -> Self {
        InstructionError::ExecutionError(err)
    }
}

/// Execute a decoded instruction
///
/// This is the main instruction dispatcher that routes instructions
/// to their specific implementations.
///
/// # Arguments
/// * `cpu` - Mutable reference to CPU state
/// * `memory` - Mutable reference to memory system
/// * `instruction` - The decoded instruction to execute
///
/// # Returns
/// * `Ok(())` - Instruction executed successfully
/// * `Err(InstructionError)` - If execution fails
///
/// # Example
/// ```rust
// / use web_x86_core::cpu::CPU;
// / use web_x86_core::memory::Memory;
// / use web_x86_core::decoder::{Instruction, Opcode, Operand};
// / use web_x86_core::cpu::RegisterName;
// / use web_x86_core::instructions::execute;
// /
// / let mut cpu = CPU::new();
// / let mut memory = Memory::new(0x1000000);
// /
// / // Set up test
// / cpu.registers.eax = 0x12345678;
// / cpu.registers.esp = 0x00FF0000;
// /
// / // Create PUSH EAX instruction
// / let instruction = Instruction {
// /     opcode: Opcode::PUSH,
// /     dest: None,
// /     src: Some(Operand::Register(RegisterName::EAX)),
// /     length: 1,
// / };
// /
// / // Execute instruction
// / execute(&mut cpu, &mut memory, &instruction).unwrap();
/// ```
pub fn execute(
    cpu: &mut CPU,
    memory: &mut Memory,
    instruction: &Instruction,
) -> Result<(), InstructionError> {
    match instruction.opcode {
        Opcode::POP => {
            pop::execute(cpu, memory, instruction);
            Ok(())
        } // Add more instructions here as we implement them
          // Opcode::ADD => add::execute(cpu, memory, instruction)?,
          // Opcode::MOV => mov::execute(cpu, memory, instruction)?,
          // etc.
    }
}
