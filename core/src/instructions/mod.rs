//! Instructions Module
//!
//! This module contains all instruction implementations and the
//! instruction execution dispatcher.

use std::fmt;

pub mod pop;
pub mod push;
pub mod call;
pub mod sub;
pub mod add;
pub mod mov;
pub mod jmp;
pub mod ret;

use crate::cpu::CPU;
use crate::decoder::{Instruction, Opcode};
use crate::memory::Memory;

/// Instruction execution errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstructionError {
    /// Unsupported instruction
    UnsupportedInstruction(Opcode),

    /// Execution error from specific instructions
    PopError(pop::ExecutionError),
    PushError(push::ExecutionError),
    MovError(String),
    SubError(sub::ExecutionError),
    AddError(add::ExecutionError),
    /// JMP instruction specific errors
    JmpError(String),
    /// Execution error from RET
    RetError(ret::ExecutionError),
}

impl fmt::Display for InstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionError::UnsupportedInstruction(opcode) => {
                write!(f, "Unsupported instruction: {}", opcode)
            },
            InstructionError::PopError(err) => {
                write!(f, "POP error: {}", err)
            },
            InstructionError::PushError(err) => {
                write!(f, "PUSH error: {}", err)
            },
            InstructionError::MovError(msg) => {
                write!(f, "MOV error: {}", msg)
            },
            InstructionError::SubError(err) => {
                write!(f, "SUB error: {:?}", err)
            },
            InstructionError::AddError(err) => {
                write!(f, "ADD error: {:?}", err)
            },
            InstructionError::JmpError(msg) => {
                write!(f, "JMP error: {}", msg)
            },
            InstructionError::RetError(err) => {
                write!(f, "RET error: {}", err)
            },
        }
    }
}

impl std::error::Error for InstructionError {}

impl From<pop::ExecutionError> for InstructionError {
    fn from(err: pop::ExecutionError) -> Self {
        InstructionError::PopError(err)
    }
}

impl From<push::ExecutionError> for InstructionError {
    fn from(err: push::ExecutionError) -> Self {
        InstructionError::PushError(err)
    }
}

impl From<mov::ExecutionError> for InstructionError {
    fn from(err: mov::ExecutionError) -> Self {
        // Use Debug so mov::ExecutionError doesn't need Display/Clone/Eq
        InstructionError::MovError(format!("{:?}", err))
    }
}

impl From<sub::ExecutionError> for InstructionError {
    fn from(err: sub::ExecutionError) -> Self {
        InstructionError::SubError(err)
    }
}

impl From<add::ExecutionError> for InstructionError {
    fn from(err: add::ExecutionError) -> Self {
        InstructionError::AddError(err)
    }
}

impl From<String> for InstructionError {
    fn from(err: String) -> Self {
        InstructionError::JmpError(err)
    }
}

impl From<ret::ExecutionError> for InstructionError {
    fn from(err: ret::ExecutionError) -> Self {
        InstructionError::RetError(err)
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
            pop::execute(cpu, memory, instruction)?;
            Ok(())
        },
        Opcode::PUSH => {
            push::execute(cpu, memory, instruction)?;
            Ok(())
        },
        Opcode::MOV => {
            mov::execute(cpu, memory, instruction)?;
            Ok(())
        },
        Opcode::SUB => {
            sub::execute(cpu, memory, instruction)?;
            Ok(())
        },
        Opcode::ADD => {
            add::add(cpu, memory, instruction)?;
            Ok(())
        },
        Opcode::JMP => {
            jmp::execute(cpu, memory, instruction)?;
            Ok(())
        },
        Opcode::RET => {
            ret::execute(cpu, memory, instruction)?;
            Ok(())
        },
        Opcode::CALL => {
            call::execute(cpu, memory, instruction)?;
            Ok(())
        },
    }
}
