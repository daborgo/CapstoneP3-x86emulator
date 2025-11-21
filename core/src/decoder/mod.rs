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
    // POP instruction - pop register from top of stack
    POP,
    /// PUSH instruction - push register onto stack
    PUSH,
    // CALL instruction - call from memory
    CALL,
    /// MOV instruction - move from source to destination
    MOV,
    /// SUB instruction - subtract source from destination
    SUB,
    /// JMP instruction - jump to target location
    JMP,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::POP => write!(f, "POP"),
            Opcode::PUSH => write!(f, "PUSH"),
            Opcode::CALL => write!(f, "CALL"),
            Opcode::MOV  => write!(f, "MOV"),
            Opcode::SUB => write!(f, "SUB"),
            Opcode::JMP => write!(f, "JMP"),
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

        // PUSH register instructions (0x50-0x57)
        0x50 => Ok(Opcode::PUSH),  // PUSH EAX
        0x51 => Ok(Opcode::PUSH),  // PUSH ECX
        0x52 => Ok(Opcode::PUSH),  // PUSH EDX
        0x53 => Ok(Opcode::PUSH),  // PUSH EBX
        0x54 => Ok(Opcode::PUSH),  // PUSH ESP
        0x55 => Ok(Opcode::PUSH),  // PUSH EBP
        0x56 => Ok(Opcode::PUSH),  // PUSH ESI
        0x57 => Ok(Opcode::PUSH),  // PUSH EDI
      
        // CALL Instruction
        0xE8 => Ok(Opcode::CALL),

        // MOV instruction
        0xB8 => Ok(Opcode::MOV), // MOV EAX
        0xB9 => Ok(Opcode::MOV), // MOV ECX
        0xBA => Ok(Opcode::MOV), // MOV EDX
        0xBB => Ok(Opcode::MOV), // MOV EBX
        0xBC => Ok(Opcode::MOV), // MOV ESP
        0xBD => Ok(Opcode::MOV), // MOV EBP
        0xBE => Ok(Opcode::MOV), // MOV ESI
        0xBF => Ok(Opcode::MOV), // MOV EDI

        
        // JMP instructions
        0xEB => Ok(Opcode::JMP),  // Short JMP rel8
        0xE9 => Ok(Opcode::JMP),  // Near JMP rel32
        0xFF => Ok(Opcode::JMP),  // Indirect JMP r/m32
        
        // SUB instructions
        0x28 => Ok(Opcode::SUB),  // SUB r/m8, r8
        0x29 => Ok(Opcode::SUB),  // SUB r/m32, r32
        0x2A => Ok(Opcode::SUB),  // SUB r8, r/m8
        0x2B => Ok(Opcode::SUB),  // SUB r32, r/m32
        0x2C => Ok(Opcode::SUB),  // SUB AL, imm8
        0x2D => Ok(Opcode::SUB),  // SUB EAX, imm32
        0x80 => Ok(Opcode::SUB),  // SUB r/m8, imm8 (when reg field of ModRM = /5)
        0x81 => Ok(Opcode::SUB),  // SUB r/m32, imm32 (when reg field of ModRM = /5)
        0x83 => Ok(Opcode::SUB),  // SUB r/m32, imm8 (when reg field of ModRM = /5)
        
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

fn mov_imm_register(opcode_byte: u8) -> Result<crate::cpu::RegisterName, DecodeError> {
    match opcode_byte {
        0xB8 => Ok(crate::cpu::RegisterName::EAX),
        0xB9 => Ok(crate::cpu::RegisterName::ECX),
        0xBA => Ok(crate::cpu::RegisterName::EDX),
        0xBB => Ok(crate::cpu::RegisterName::EBX),
        0xBC => Ok(crate::cpu::RegisterName::ESP),
        0xBD => Ok(crate::cpu::RegisterName::EBP),
        0xBE => Ok(crate::cpu::RegisterName::ESI),
        0xBF => Ok(crate::cpu::RegisterName::EDI),

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
        },
        Opcode::CALL => {
            if bytes.len() < 5 {
                return Err(DecodeError::InsufficientBytes);
            }
            let disp = u32::from_le_bytes(bytes[1..5].try_into().unwrap());
            Ok(Instruction {
                opcode,
                dest: None,
                src: Some(Operand::Immediate(disp)),
                length: 5,
            })
        },
        
         Opcode::MOV => {
            // Handle MOV imm32 -> reg (opcodes 0xB8 .. 0xBF)
            // Instruction layout: opcode (1 byte) + imm32 (4 bytes)
            if bytes.len() < 5 {
                return Err(DecodeError::InsufficientBytes);
            }

            let dest_reg = mov_imm_register(opcode_byte)?;

            // Little-endian immediate u32 from bytes[1..5]
            let imm = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);

            Ok(Instruction {
                opcode,
                dest: Some(Operand::Register(dest_reg)),
                src: Some(Operand::Immediate(imm)),
                length: 5,
            })
        },
      
        Opcode::JMP => {
            match opcode_byte {
                0xEB => {
                    // Short JMP (8-bit displacement)
                    if bytes.len() < 2 {
                        return Err(DecodeError::InsufficientBytes);
                    }
                    Ok(Instruction {
                        opcode,
                        dest: Some(Operand::Immediate((bytes[1] as i8) as i32 as u32)),
                        src: None,
                        length: 2,
                    })
                },
                0xE9 => {
                    // Near JMP (32-bit displacement)
                    if bytes.len() < 5 {
                        return Err(DecodeError::InsufficientBytes);
                    }
                    let displacement = ((bytes[4] as u32) << 24) |
                                     ((bytes[3] as u32) << 16) |
                                     ((bytes[2] as u32) << 8) |
                                     (bytes[1] as u32);
                    Ok(Instruction {
                        opcode,
                        dest: Some(Operand::Immediate(displacement)),
                        src: None,
                        length: 5,
                    })
                },
                _ => Err(DecodeError::UnknownOpcode(opcode_byte)),
            }
        },
        Opcode::SUB => {
            // For now, we'll implement a simple case: SUB between registers
            // This will need to be expanded based on the ModR/M byte and other forms
            if bytes.len() < 2 {
                return Err(DecodeError::InsufficientBytes);
            }
            
            // For this example, we'll assume it's a register-to-register SUB
            // In a full implementation, you'd need to parse the ModR/M byte properly
            let dest_reg = match bytes[1] >> 3 & 0x7 {
                0 => crate::cpu::RegisterName::EAX,
                1 => crate::cpu::RegisterName::ECX,
                2 => crate::cpu::RegisterName::EDX,
                3 => crate::cpu::RegisterName::EBX,
                4 => crate::cpu::RegisterName::ESP,
                5 => crate::cpu::RegisterName::EBP,
                6 => crate::cpu::RegisterName::ESI,
                7 => crate::cpu::RegisterName::EDI,
                _ => unreachable!(),
            };

            let src_reg = match bytes[1] & 0x7 {
                0 => crate::cpu::RegisterName::EAX,
                1 => crate::cpu::RegisterName::ECX,
                2 => crate::cpu::RegisterName::EDX,
                3 => crate::cpu::RegisterName::EBX,
                4 => crate::cpu::RegisterName::ESP,
                5 => crate::cpu::RegisterName::EBP,
                6 => crate::cpu::RegisterName::ESI,
                7 => crate::cpu::RegisterName::EDI,
                _ => unreachable!(),
            };
            
            Ok(Instruction {
                opcode,
                dest: Some(Operand::Register(dest_reg)),
                src: Some(Operand::Register(src_reg)),
                length: 2,  // opcode byte + ModR/M byte
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
        assert!(parse_opcode(0x10).is_err());  // Use a different invalid opcode
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

    #[test]
    fn test_decode_mov_imm_to_eax() {
        // MOV EAX, 0xDEADBEEF -> opcode 0xB8 followed by imm32 little-endian
        let bytes = [0xB8, 0xEF, 0xBE, 0xAD, 0xDE];
        let instruction = decode(&bytes).unwrap();

        assert_eq!(instruction.opcode, Opcode::MOV);
        assert_eq!(instruction.dest, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instruction.src, Some(Operand::Immediate(0xDEADBEEF)));
        assert_eq!(instruction.length, 5);
    }

    fn test_decode_sub_register() {
        // Example: SUB EAX, EBX (register-to-register form)
        let bytes = [0x2B, 0xC3];  // 0x2B is SUB r32,r/m32, 0xC3 is ModR/M byte for EAX,EBX
        let instruction = decode(&bytes).unwrap();
        
        assert_eq!(instruction.opcode, Opcode::SUB);
        assert_eq!(instruction.dest, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instruction.src, Some(Operand::Register(RegisterName::EBX)));
        assert_eq!(instruction.length, 2);
    }
}

//         assert!(formatted.contains("POP"));
//         assert!(formatted.contains("EAX"));
//         assert!(formatted.contains("1 bytes"));
//     }
// }
