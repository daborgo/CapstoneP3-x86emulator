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
pub mod and;
pub mod or;
pub mod mov;
pub mod jmp;
pub mod ret;
pub mod mul;
pub mod and;
pub mod or;
pub mod shift;
pub mod cmp;
pub mod instruction_error_tests;

use crate::cpu::CPU;
use crate::decoder::{Instruction, Opcode, Operand, resolve_memory};
use crate::memory::Memory;

/// Instruction execution errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstructionError {
    UnsupportedInstruction(Opcode),
    PopError(pop::ExecutionError),
    PushError(push::ExecutionError),
    MovError(String),
    SubError(sub::ExecutionError),
    AddError(add::ExecutionError),
    AndError(and::ExecutionError),
    OrError(or::ExecutionError),
    CmpError(cmp::ExecutionError),
    /// JMP instruction specific errors
    JmpError(String),
    RetError(ret::ExecutionError),
    MulError(mul::ExecutionError),
    AndError(and::ExecutionError),
    OrError(or::ExecutionError),
    ShiftError(shift::ExecutionError),
    CmpError(cmp::ExecutionError),
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
            InstructionError::AndError(err) => {
                write!(f, "AND error: {}", err)
            },
            InstructionError::OrError(err) => {
                write!(f, "OR error: {}", err)
            },
            InstructionError::CmpError(err) => {
                write!(f, "CMP error: {}", err)
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

impl From<pop::ExecutionError>   for InstructionError { fn from(e: pop::ExecutionError)   -> Self { InstructionError::PopError(e) } }
impl From<push::ExecutionError>  for InstructionError { fn from(e: push::ExecutionError)  -> Self { InstructionError::PushError(e) } }
impl From<sub::ExecutionError>   for InstructionError { fn from(e: sub::ExecutionError)   -> Self { InstructionError::SubError(e) } }
impl From<add::ExecutionError>   for InstructionError { fn from(e: add::ExecutionError)   -> Self { InstructionError::AddError(e) } }
impl From<ret::ExecutionError>   for InstructionError { fn from(e: ret::ExecutionError)   -> Self { InstructionError::RetError(e) } }
impl From<mul::ExecutionError>   for InstructionError { fn from(e: mul::ExecutionError)   -> Self { InstructionError::MulError(e) } }
impl From<and::ExecutionError>   for InstructionError { fn from(e: and::ExecutionError)   -> Self { InstructionError::AndError(e) } }
impl From<or::ExecutionError>    for InstructionError { fn from(e: or::ExecutionError)    -> Self { InstructionError::OrError(e) } }
impl From<shift::ExecutionError> for InstructionError { fn from(e: shift::ExecutionError) -> Self { InstructionError::ShiftError(e) } }
impl From<cmp::ExecutionError>   for InstructionError { fn from(e: cmp::ExecutionError)   -> Self { InstructionError::CmpError(e) } }
impl From<mov::ExecutionError>   for InstructionError {
    fn from(e: mov::ExecutionError) -> Self { InstructionError::MovError(format!("{:?}", e)) }
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

impl From<and::ExecutionError> for InstructionError {
    fn from(err: and::ExecutionError) -> Self {
        InstructionError::AndError(err)
    }
}

impl From<or::ExecutionError> for InstructionError {
    fn from(err: or::ExecutionError) -> Self {
        InstructionError::OrError(err)
    }
}

impl From<cmp::ExecutionError> for InstructionError {
    fn from(err: cmp::ExecutionError) -> Self {
        InstructionError::CmpError(err)
    }
}

impl From<String> for InstructionError {
    fn from(s: String) -> Self { InstructionError::JmpError(s) }
}

// ─── Memory sentinel resolver for MOV ────────────────────────────────────────
// Translates an Instruction that may contain Memory sentinels into one with
// resolved absolute addresses, so the existing mov::execute can work unchanged.

fn resolve_instruction_memory(instr: &Instruction, cpu: &CPU) -> Instruction {
    let resolve = |op: Option<Operand>| -> Option<Operand> {
        match op {
            Some(Operand::Memory(sentinel)) if sentinel >= 0x4000_0000 => {
                Some(Operand::Memory(resolve_memory(sentinel, cpu)))
            }
            other => other,
        }
    };
    Instruction {
        opcode: instr.opcode,
        dest: resolve(instr.dest),
        src: resolve(instr.src),
        length: instr.length,
    }
}

// ─── Main dispatcher ──────────────────────────────────────────────────────────

pub fn execute(
    cpu: &mut CPU,
    memory: &mut Memory,
    instruction: &Instruction,
) -> Result<(), InstructionError> {
    match instruction.opcode {
        Opcode::POP  => { pop::execute(cpu, memory, instruction)?; Ok(()) }
        Opcode::PUSH => { push::execute(cpu, memory, instruction)?; Ok(()) }
        Opcode::MOV  => {
            let resolved = resolve_instruction_memory(instruction, cpu);
            mov::execute(cpu, memory, &resolved)?;
            Ok(())
        }
        Opcode::SUB  => { sub::execute(cpu, memory, instruction)?; Ok(()) }
        Opcode::ADD  => { add::add(cpu, memory, instruction)?; Ok(()) }
        Opcode::JMP  => { jmp::execute(cpu, memory, instruction)?; Ok(()) }
        Opcode::RET  => { ret::execute(cpu, memory, instruction)?; Ok(()) }
        Opcode::CALL => { call::execute(cpu, memory, instruction)?; Ok(()) }
        Opcode::MUL  => { mul::execute_mul(cpu, memory, instruction)?; Ok(()) }
        Opcode::IMUL => { mul::execute_imul(cpu, memory, instruction)?; Ok(()) }
        Opcode::IDIV => { mul::execute_idiv(cpu, memory, instruction)?; Ok(()) }
        Opcode::CDQ  => { mul::execute_cdq(cpu, memory, instruction)?; Ok(()) }
        Opcode::AND  => { and::execute(cpu, memory, instruction)?; Ok(()) }
        Opcode::OR   => { or::execute(cpu, memory, instruction)?; Ok(()) }
        Opcode::SHL  => { shift::execute_shl(cpu, memory, instruction)?; Ok(()) }
        Opcode::SHR  => { shift::execute_shr(cpu, memory, instruction)?; Ok(()) }
        Opcode::SAR  => { shift::execute_sar(cpu, memory, instruction)?; Ok(()) }
        Opcode::CMP  => { cmp::execute(cpu, memory, instruction)?; Ok(()) }
        Opcode::JE | Opcode::JNE | Opcode::JL | Opcode::JGE | Opcode::JLE | Opcode::JG => {
            execute_jcc(cpu, instruction)
        }
    }
}

/// Execute a conditional jump instruction
fn execute_jcc(cpu: &mut CPU, instruction: &Instruction) -> Result<(), InstructionError> {
    let taken = match instruction.opcode {
        Opcode::JE  => cpu.flags.zf,
        Opcode::JNE => !cpu.flags.zf,
        Opcode::JL  => cpu.flags.sf != cpu.flags.of,
        Opcode::JGE => cpu.flags.sf == cpu.flags.of,
        Opcode::JLE => cpu.flags.zf || (cpu.flags.sf != cpu.flags.of),
        Opcode::JG  => !cpu.flags.zf && (cpu.flags.sf == cpu.flags.of),
        _ => return Err(InstructionError::UnsupportedInstruction(instruction.opcode)),
    };

    if taken {
        if let Some(Operand::Immediate(disp)) = instruction.dest {
            let disp = disp as i8;
            let new_ip = (cpu.registers.eip as i32)
                .wrapping_add(instruction.length as i32)
                .wrapping_add(disp as i32);
            cpu.registers.eip = new_ip as u32;
        } else {
            return Err(InstructionError::JmpError("Missing displacement for conditional jump".to_string()));
        }
    } else {
        cpu.registers.advance_ip(instruction.length as u32);
    }

    Ok(())
}
