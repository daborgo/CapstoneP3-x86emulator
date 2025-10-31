//! Instruction Decoder Module
//!
//! This module handles parsing raw instruction bytes into structured
//! instruction representations that can be executed by the CPU.

use std::fmt;

/// Decoder errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Unknown or unsupported opcode
    UnknownOpcode(u8),

    /// Invalid instruction format
    InvalidFormat,

    /// Not enough bytes to decode instruction
    InsufficientBytes,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::UnknownOpcode(opcode) => {
                write!(f, "Unknown opcode: 0x{:02X}", opcode)
            }
            DecodeError::InvalidFormat => {
                write!(f, "Invalid instruction format")
            }
            DecodeError::InsufficientBytes => {
                write!(f, "Not enough bytes to decode instruction")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Supported instruction opcodes
///
/// This can be expanded as we add more instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    POP,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::POP => write!(f, "POP"),
        }
    }
}

/// Operand types for instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    /// Register operand (EAX, EBX, etc.)
    Register(crate::cpu::RegisterName),

    /// Immediate value (constant)
    Immediate(u32),

    /// Memory address
    Memory(u32),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{:?}", reg),
            Operand::Immediate(val) => write!(f, "0x{:08X}", val),
            Operand::Memory(addr) => write!(f, "[0x{:08X}]", addr),
        }
    }
}

/// Decoded instruction structure
///
/// This represents a fully decoded instruction ready for execution.
/// It contains all the information needed to execute the instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// The operation to perform
    pub opcode: Opcode,

    /// Primary operand (destination for most instructions)
    pub dest: Option<Operand>,

    /// Secondary operand (source for most instructions)
    pub src: Option<Operand>,

    /// Length of this instruction in bytes
    pub length: u8,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.opcode)?;

        if let Some(dest) = self.dest {
            write!(f, " {}", dest)?;
        }

        if let Some(src) = self.src {
            write!(f, ", {}", src)?;
        }

        write!(f, " ({} bytes)", self.length)
    }
}

/// Parse a single byte opcode
///
/// This function maps raw opcode bytes to our Opcode enum.
///
/// # Arguments
/// * `opcode_byte` - The first byte of the instruction
///
/// # Returns
/// * `Ok(Opcode)` - The decoded opcode
/// * `Err(DecodeError)` - If the opcode is unknown

pub fn parse_opcode(opcode_byte: u8) -> Result<Opcode, DecodeError> {
    match opcode_byte {
        //POP register instructions
        0x58..=0x5F => Ok(Opcode::POP),

        _ => Err(DecodeError::UnknownOpcode(opcode_byte)),
    }
}

//register for pop
pub fn get_pop_register(opcode_byte: u8) -> Result<crate::cpu::RegisterName, DecodeError> {
    match opcode_byte {
        0x58 => Ok(crate::cpu::RegisterName::EAX),
        0x59 => Ok(crate::cpu::RegisterName::ECX),
        0x5A => Ok(crate::cpu::RegisterName::EDX),
        0x5B => Ok(crate::cpu::RegisterName::EBX),
        0x5C => Ok(crate::cpu::RegisterName::ESP),
        0x5D => Ok(crate::cpu::RegisterName::EBP),
        0x5E => Ok(crate::cpu::RegisterName::ESI),
        0x5F => Ok(crate::cpu::RegisterName::EDI),
        _ => Err(DecodeError::UnknownOpcode(opcode_byte)),
    }
}

/// Decode instruction bytes into a structured Instruction
///
/// This is the main decoding function that takes raw bytes
/// and produces a structured instruction ready for execution.
///
/// # Arguments
/// * `bytes` - Slice of instruction bytes starting at EIP
///
/// # Returns
/// * `Ok(Instruction)` - The decoded instruction
/// * `Err(DecodeError)` - If decoding fails
///
/// # Example
/// ```rust
//use web_x86_core::decoder::{decode, Opcode};

pub fn decode(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    // Check if we have at least one byte
    if bytes.is_empty() {
        return Err(DecodeError::InsufficientBytes);
    }

    let opcode_byte = bytes[0];

    // Parse the opcode
    let opcode = parse_opcode(opcode_byte)?;

    match opcode {
        Opcode::POP => {
            let register = get_pop_register(opcode_byte)?;
            Ok(Instruction {
                opcode,
                dest: Some(Operand::Register(register)),
                src: None,
                length: 1,
            })
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::cpu::RegisterName;

//     #[test]
//     fn test_parse_opcode_unknown() {
//         assert!(parse_opcode(0x00).is_err());
//         assert!(parse_opcode(0xFF).is_err());
//     }

//     #[test]
//     fn test_decode_empty_bytes() {
//         let bytes = [];
//         assert!(decode(&bytes).is_err());
//     }

//     #[test]
//     fn test_instruction_display() {
//         let bytes = [0x50];
//         let instruction = decode(&bytes).unwrap();
//         let formatted = format!("{}", instruction);

//         assert!(formatted.contains("POP"));
//         assert!(formatted.contains("EAX"));
//         assert!(formatted.contains("1 bytes"));
//     }
// }
