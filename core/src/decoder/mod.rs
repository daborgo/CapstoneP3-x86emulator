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
            },
            DecodeError::InvalidFormat => {
                write!(f, "Invalid instruction format")
            },
            DecodeError::InsufficientBytes => {
                write!(f, "Not enough bytes to decode instruction")
            },
        }
    }
}

impl std::error::Error for DecodeError {}

/// Supported instruction opcodes
/// 
/// For now, we'll start with a minimal set focusing on PUSH.
/// This can be expanded as we add more instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    /// PUSH instruction - push register onto stack
    PUSH,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::PUSH => write!(f, "PUSH"),
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
/// For now, we only support PUSH instructions.
/// 
/// # Arguments
/// * `opcode_byte` - The first byte of the instruction
/// 
/// # Returns
/// * `Ok(Opcode)` - The decoded opcode
/// * `Err(DecodeError)` - If the opcode is unknown
pub fn parse_opcode(opcode_byte: u8) -> Result<Opcode, DecodeError> {
    match opcode_byte {
        // PUSH register instructions (0x50-0x57)
        0x50 => Ok(Opcode::PUSH),  // PUSH EAX
        0x51 => Ok(Opcode::PUSH),  // PUSH ECX
        0x52 => Ok(Opcode::PUSH),  // PUSH EDX
        0x53 => Ok(Opcode::PUSH),  // PUSH EBX
        0x54 => Ok(Opcode::PUSH),  // PUSH ESP
        0x55 => Ok(Opcode::PUSH),  // PUSH EBP
        0x56 => Ok(Opcode::PUSH),  // PUSH ESI
        0x57 => Ok(Opcode::PUSH),  // PUSH EDI
        
        _ => Err(DecodeError::UnknownOpcode(opcode_byte)),
    }
}

/// Get the register for a PUSH opcode
/// 
/// PUSH instructions use a simple encoding where the opcode
/// directly indicates which register to push.
/// 
/// # Arguments
/// * `opcode_byte` - The opcode byte (0x50-0x57)
/// 
/// # Returns
/// * `Ok(RegisterName)` - The register to push
/// * `Err(DecodeError)` - If the opcode is invalid
pub fn get_push_register(opcode_byte: u8) -> Result<crate::cpu::RegisterName, DecodeError> {
    match opcode_byte {
        0x50 => Ok(crate::cpu::RegisterName::EAX),
        0x51 => Ok(crate::cpu::RegisterName::ECX),
        0x52 => Ok(crate::cpu::RegisterName::EDX),
        0x53 => Ok(crate::cpu::RegisterName::EBX),
        0x54 => Ok(crate::cpu::RegisterName::ESP),
        0x55 => Ok(crate::cpu::RegisterName::EBP),
        0x56 => Ok(crate::cpu::RegisterName::ESI),
        0x57 => Ok(crate::cpu::RegisterName::EDI),
        
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
/// use web_x86_core::decoder::{decode, Opcode};
/// 
/// // PUSH EAX instruction
/// let bytes = [0x50];
/// let instruction = decode(&bytes).unwrap();
/// assert_eq!(instruction.opcode, Opcode::PUSH);
/// ```
pub fn decode(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    // Check if we have at least one byte
    if bytes.is_empty() {
        return Err(DecodeError::InsufficientBytes);
    }
    
    let opcode_byte = bytes[0];
    
    // Parse the opcode
    let opcode = parse_opcode(opcode_byte)?;
    
    match opcode {
        Opcode::PUSH => {
            // PUSH register instructions are 1 byte
            let register = get_push_register(opcode_byte)?;
            
            Ok(Instruction {
                opcode,
                dest: None,  // PUSH doesn't have a destination
                src: Some(Operand::Register(register)),
                length: 1,
            })
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::RegisterName;
    
    #[test]
    fn test_parse_opcode_push() {
        assert_eq!(parse_opcode(0x50), Ok(Opcode::PUSH));
        assert_eq!(parse_opcode(0x51), Ok(Opcode::PUSH));
        assert_eq!(parse_opcode(0x57), Ok(Opcode::PUSH));
    }
    
    #[test]
    fn test_parse_opcode_unknown() {
        assert!(parse_opcode(0x00).is_err());
        assert!(parse_opcode(0xFF).is_err());
    }
    
    #[test]
    fn test_get_push_register() {
        assert_eq!(get_push_register(0x50), Ok(RegisterName::EAX));
        assert_eq!(get_push_register(0x51), Ok(RegisterName::ECX));
        assert_eq!(get_push_register(0x52), Ok(RegisterName::EDX));
        assert_eq!(get_push_register(0x53), Ok(RegisterName::EBX));
        assert_eq!(get_push_register(0x54), Ok(RegisterName::ESP));
        assert_eq!(get_push_register(0x55), Ok(RegisterName::EBP));
        assert_eq!(get_push_register(0x56), Ok(RegisterName::ESI));
        assert_eq!(get_push_register(0x57), Ok(RegisterName::EDI));
    }
    
    #[test]
    fn test_decode_push_eax() {
        let bytes = [0x50];
        let instruction = decode(&bytes).unwrap();
        
        assert_eq!(instruction.opcode, Opcode::PUSH);
        assert_eq!(instruction.dest, None);
        assert_eq!(instruction.src, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instruction.length, 1);
    }
    
    #[test]
    fn test_decode_push_ebx() {
        let bytes = [0x53];
        let instruction = decode(&bytes).unwrap();
        
        assert_eq!(instruction.opcode, Opcode::PUSH);
        assert_eq!(instruction.src, Some(Operand::Register(RegisterName::EBX)));
        assert_eq!(instruction.length, 1);
    }
    
    #[test]
    fn test_decode_empty_bytes() {
        let bytes = [];
        assert!(decode(&bytes).is_err());
    }
    
    #[test]
    fn test_instruction_display() {
        let bytes = [0x50];
        let instruction = decode(&bytes).unwrap();
        let formatted = format!("{}", instruction);
        
        assert!(formatted.contains("PUSH"));
        assert!(formatted.contains("EAX"));
        assert!(formatted.contains("1 bytes"));
    }
}

