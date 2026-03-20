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
    /// ADD instruction - add source to destination
    ADD,
    /// AND instruction - bitwise and source and destination
    AND,
    /// JMP instruction - jump to target location
    JMP,
    /// RET instruction - return from function
    RET,

    CMP,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::POP => write!(f, "POP"),
            Opcode::PUSH => write!(f, "PUSH"),
            Opcode::CALL => write!(f, "CALL"),
            Opcode::MOV  => write!(f, "MOV"),
            Opcode::SUB => write!(f, "SUB"),
            Opcode::ADD => write!(f, "ADD"),
            Opcode::AND => write!(f, "AND"),
            Opcode::JMP => write!(f, "JMP"),
            Opcode::RET => write!(f, "RET"),
            Opcode::CMP => write!(f, "CMP"),
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
        0x89 => Ok(Opcode::MOV), // MOV r/m32, r32
        0x8B => Ok(Opcode::MOV), // MOV r32, r/m32

        
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
        0x80 => Ok(Opcode::SUB),  // SUB r/m8, imm8 (when reg field of ModRM = /5) - also used for ADD with /0
        // 0x81 is a group opcode; route to ADD handler and inspect ModR/M reg field later.
        0x81 => Ok(Opcode::ADD),
        0x83 => Ok(Opcode::SUB),  // SUB r/m32, imm8 (when reg field of ModRM = /5)
        
        // ADD instructions
        0x00 => Ok(Opcode::ADD),  // ADD r/m8, r8
        0x01 => Ok(Opcode::ADD),  // ADD r/m32, r32
        0x02 => Ok(Opcode::ADD),  // ADD r8, r/m8
        0x03 => Ok(Opcode::ADD),  // ADD r32, r/m32
        0x04 => Ok(Opcode::ADD),  // ADD AL, imm8
        0x05 => Ok(Opcode::ADD),  // ADD EAX, imm32

        // AND instructions
        0x21 => Ok(Opcode::AND),  // AND r/m32, r32
        0x23 => Ok(Opcode::AND),  // AND r32, r/m32
        
        // RET instruction
        0xC3 => Ok(Opcode::RET),  // Near return

        // CMP instructions (register forms)
        0x38 => Ok(Opcode::CMP),  // CMP r/m8, r8
        0x39 => Ok(Opcode::CMP),  // CMP r/m32, r32
        0x3A => Ok(Opcode::CMP),  // CMP r8, r/m8
        0x3B => Ok(Opcode::CMP),  // CMP r32, r/m32
        
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
        Opcode::PUSH => {
            let register = get_push_register(opcode_byte)?;
            Ok(Instruction {
                opcode,
                dest: None,
                src: Some(Operand::Register(register)),
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
            match opcode_byte {
                // MOV imm32 -> reg (opcodes 0xB8 .. 0xBF)
                0xB8..=0xBF => {
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
                }
                // MOV r/m32, r32 (0x89) and MOV r32, r/m32 (0x8B)
                0x89 | 0x8B => {
                    if bytes.len() < 2 {
                        return Err(DecodeError::InsufficientBytes);
                    }

                    let modrm = bytes[1];
                    let mod_bits = modrm >> 6;

                    // Only handle register-to-register (mod = 11) for now
                    if mod_bits != 0b11 {
                        return Err(DecodeError::InvalidFormat);
                    }

                    let reg_field = match (modrm >> 3) & 0x7 {
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

                    let rm_field = match modrm & 0x7 {
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

                    let (dest_reg, src_reg) = if opcode_byte == 0x89 {
                        // MOV r/m32, r32
                        (rm_field, reg_field)
                    } else {
                        // MOV r32, r/m32
                        (reg_field, rm_field)
                    };

                    Ok(Instruction {
                        opcode,
                        dest: Some(Operand::Register(dest_reg)),
                        src: Some(Operand::Register(src_reg)),
                        length: 2,
                    })
                }
                _ => Err(DecodeError::UnknownOpcode(opcode_byte)),
            }
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
            match opcode_byte {
                0x29 | 0x2B => {
                    // SUB r/m32, r32 (0x29) or SUB r32, r/m32 (0x2B)
                    if bytes.len() < 2 {
                        return Err(DecodeError::InsufficientBytes);
                    }

                    let modrm = bytes[1];
                    let mod_bits = modrm >> 6;

                    // Only handle register-to-register (mod = 11)
                    if mod_bits != 0b11 {
                        return Err(DecodeError::InvalidFormat);
                    }

                    let reg_field = match (modrm >> 3) & 0x7 {
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

                    let rm_field = match modrm & 0x7 {
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

                    let (dest_reg, src_reg) = if opcode_byte == 0x29 {
                        (rm_field, reg_field)
                    } else {
                        (reg_field, rm_field)
                    };

                    Ok(Instruction {
                        opcode,
                        dest: Some(Operand::Register(dest_reg)),
                        src: Some(Operand::Register(src_reg)),
                        length: 2,  // opcode byte + ModR/M byte
                    })
                }
                _ => Err(DecodeError::InvalidFormat),
            }
        },
        Opcode::ADD => {
            // Handle ADD instructions similar to SUB
            match opcode_byte {
                0x01 => {
                    // ADD r/m32, r32: opcode 0x01 + ModR/M byte
                    if bytes.len() < 2 {
                        return Err(DecodeError::InsufficientBytes);
                    }
                    
                    let modrm = bytes[1];
                    let mod_bits = modrm >> 6;
                    
                    // For simplicity, only handle register-to-register (mod = 11)
                    if mod_bits == 0b11 {
                        let src_reg = match (modrm >> 3) & 0x7 {
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
                        
                        let dest_reg = match modrm & 0x7 {
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
                            length: 2,
                        })
                    } else {
                        Err(DecodeError::InvalidFormat)
                    }
                },
                0x81 => {
                    // 0x81 is a group opcode: reg field selects the actual operation
                    // /0 = ADD, /4 = AND, /5 = SUB, /7 = CMP
                    if bytes.len() < 6 {
                        return Err(DecodeError::InsufficientBytes);
                    }

                    let modrm = bytes[1];
                    let mod_bits = modrm >> 6;
                    let reg_field = (modrm >> 3) & 0x7;

                    // Only handle register mode (mod = 11)
                    if mod_bits != 0b11 {
                        return Err(DecodeError::InvalidFormat);
                    }

                    let dest_reg = match modrm & 0x7 {
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

                    let imm = u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);

                    match reg_field {
                        0 => Ok(Instruction {
                            opcode: Opcode::ADD,
                            dest: Some(Operand::Register(dest_reg)),
                            src: Some(Operand::Immediate(imm)),
                            length: 6,
                        }),
                        4 => Ok(Instruction {
                            opcode: Opcode::AND,
                            dest: Some(Operand::Register(dest_reg)),
                            src: Some(Operand::Immediate(imm)),
                            length: 6,
                        }),
                        5 => Ok(Instruction {
                            opcode: Opcode::SUB,
                            dest: Some(Operand::Register(dest_reg)),
                            src: Some(Operand::Immediate(imm)),
                            length: 6,
                        }),
                        7 => Ok(Instruction {
                            opcode: Opcode::CMP,
                            dest: Some(Operand::Register(dest_reg)),
                            src: Some(Operand::Immediate(imm)),
                            length: 6,
                        }),
                        _ => Err(DecodeError::InvalidFormat),
                    }
                },
                _ => Err(DecodeError::UnknownOpcode(opcode_byte)),
            }
        },
        Opcode::AND => {
            match opcode_byte {
                0x21 | 0x23 => {
                    // AND r/m32, r32 (0x21) or AND r32, r/m32 (0x23)
                    if bytes.len() < 2 {
                        return Err(DecodeError::InsufficientBytes);
                    }

                    let modrm = bytes[1];
                    let mod_bits = modrm >> 6;

                    // Only handle register-to-register (mod = 11)
                    if mod_bits != 0b11 {
                        return Err(DecodeError::InvalidFormat);
                    }

                    let reg_field = match (modrm >> 3) & 0x7 {
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

                    let rm_field = match modrm & 0x7 {
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

                    let (dest_reg, src_reg) = if opcode_byte == 0x21 {
                        (rm_field, reg_field)
                    } else {
                        (reg_field, rm_field)
                    };

                    Ok(Instruction {
                        opcode,
                        dest: Some(Operand::Register(dest_reg)),
                        src: Some(Operand::Register(src_reg)),
                        length: 2,
                    })
                }
                _ => Err(DecodeError::InvalidFormat),
            }
        },
        Opcode::CMP => {
            // CMP (simple register-to-register form; 0x39 + ModR/M)
            if bytes.len() < 2 {
                return Err(DecodeError::InsufficientBytes);
            }

            let modrm = bytes[1];
            let mod_bits = modrm >> 6;

            // Only support register-to-register (mod = 11) for now
            if mod_bits != 0b11 {
                return Err(DecodeError::InvalidFormat);
            }

            // Use same bit extraction as SUB/ADD for consistency with encoding
            let dest_reg = match (modrm >> 3) & 0x7 {
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

            let src_reg = match modrm & 0x7 {
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
                length: 2,
            })
        },
        Opcode::RET => {
            // RET is a single byte instruction (near return)
            Ok(Instruction {
                opcode,
                dest: None,
                src: None,
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
        assert!(parse_opcode(0x06).is_err());  // Changed from 0x00 which is now ADD
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
}
