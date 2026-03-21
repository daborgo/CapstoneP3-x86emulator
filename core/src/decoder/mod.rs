//! Instruction Decoder Module
//!
//! This module handles parsing raw instruction bytes into structured
//! instruction representations that can be executed by the CPU.

use std::fmt;

/// Decoder errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    UnknownOpcode(u8),
    InvalidFormat,
    InsufficientBytes,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::UnknownOpcode(opcode) => write!(f, "Unknown opcode: 0x{:02X}", opcode),
            DecodeError::InvalidFormat => write!(f, "Invalid instruction format"),
            DecodeError::InsufficientBytes => write!(f, "Not enough bytes to decode instruction"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Supported instruction opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    POP,
    PUSH,
    CALL,
    MOV,
    SUB,
    ADD,
    JMP,
    RET,
    MUL,
    IDIV,
    CDQ,
    AND,
    OR,
    SHL,
    SHR,
    SAR,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::POP  => write!(f, "POP"),
            Opcode::PUSH => write!(f, "PUSH"),
            Opcode::CALL => write!(f, "CALL"),
            Opcode::MOV  => write!(f, "MOV"),
            Opcode::SUB  => write!(f, "SUB"),
            Opcode::ADD  => write!(f, "ADD"),
            Opcode::JMP  => write!(f, "JMP"),
            Opcode::RET  => write!(f, "RET"),
            Opcode::MUL  => write!(f, "MUL"),
            Opcode::IDIV => write!(f, "IDIV"),
            Opcode::CDQ  => write!(f, "CDQ"),
            Opcode::AND  => write!(f, "AND"),
            Opcode::OR   => write!(f, "OR"),
            Opcode::SHL  => write!(f, "SHL"),
            Opcode::SHR  => write!(f, "SHR"),
            Opcode::SAR  => write!(f, "SAR"),
        }
    }
}

/// Operand types for instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Register(crate::cpu::RegisterName),
    Immediate(u32),
    /// Absolute or encoded memory reference.
    ///
    /// Sentinel encoding used by the decoder:
    ///   0x0000_0000..0x7FFF_FFFF  – absolute address
    ///   0x8000_0000 | rm_idx      – [register] indirect  (mod=00)
    ///   0x4000_0000 | (rm_idx<<16) | (disp & 0xFFFF) – [register+disp]  (mod=01/10)
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub dest: Option<Operand>,
    pub src: Option<Operand>,
    pub length: u8,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.opcode)?;
        if let Some(dest) = self.dest { write!(f, " {}", dest)?; }
        if let Some(src) = self.src  { write!(f, ", {}", src)?; }
        write!(f, " ({} bytes)", self.length)
    }
}

// ─── Register helpers ─────────────────────────────────────────────────────────

pub fn reg_from_index(idx: u8) -> crate::cpu::RegisterName {
    match idx & 0x7 {
        0 => crate::cpu::RegisterName::EAX,
        1 => crate::cpu::RegisterName::ECX,
        2 => crate::cpu::RegisterName::EDX,
        3 => crate::cpu::RegisterName::EBX,
        4 => crate::cpu::RegisterName::ESP,
        5 => crate::cpu::RegisterName::EBP,
        6 => crate::cpu::RegisterName::ESI,
        7 => crate::cpu::RegisterName::EDI,
        _ => unreachable!(),
    }
}

/// Resolve a memory-sentinel Operand to an absolute address using current CPU registers.
/// Returns None if the operand is not a memory sentinel.
pub fn resolve_memory(operand: u32, cpu: &crate::cpu::CPU) -> u32 {
    if operand & 0x8000_0000 != 0 && operand & 0x4000_0000 == 0 {
        // [register] form: low bits = register index
        let rm_idx = (operand & 0x7) as u8;
        cpu.registers.get(reg_from_index(rm_idx))
    } else if operand & 0x4000_0000 != 0 {
        // [register + disp] form
        let rm_idx = ((operand >> 16) & 0x7) as u8;
        let disp = (operand & 0xFFFF) as i16 as i32;
        let base = cpu.registers.get(reg_from_index(rm_idx));
        base.wrapping_add(disp as u32)
    } else {
        // absolute address
        operand
    }
}

// ─── Group decoders ───────────────────────────────────────────────────────────

/// 0x80 / 0x81 / 0x83 – arithmetic/logic with immediate operand
fn decode_group1(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    if bytes.len() < 2 {
        return Err(DecodeError::InsufficientBytes);
    }
    let opcode_byte = bytes[0];
    let modrm = bytes[1];
    let mod_bits = modrm >> 6;
    let reg_field = (modrm >> 3) & 0x7;
    let rm_idx = modrm & 0x7;

    if mod_bits != 0b11 {
        return Err(DecodeError::InvalidFormat); // only reg destination for now
    }

    let dst_reg = reg_from_index(rm_idx);

    let (imm, imm_bytes): (u32, u8) = if opcode_byte == 0x83 {
        if bytes.len() < 3 { return Err(DecodeError::InsufficientBytes); }
        (bytes[2] as i8 as i32 as u32, 1)
    } else {
        if bytes.len() < 6 { return Err(DecodeError::InsufficientBytes); }
        (u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]), 4)
    };

    let opcode = match reg_field {
        0 => Opcode::ADD,
        1 => Opcode::OR,
        4 => Opcode::AND,
        5 => Opcode::SUB,
        _ => return Err(DecodeError::UnknownOpcode(opcode_byte)),
    };

    Ok(Instruction {
        opcode,
        dest: Some(Operand::Register(dst_reg)),
        src: Some(Operand::Immediate(imm)),
        length: 2 + imm_bytes,
    })
}

/// 0xF7 – MUL (/4) or IDIV (/7)
fn decode_f7(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    if bytes.len() < 2 { return Err(DecodeError::InsufficientBytes); }
    let modrm = bytes[1];
    let mod_bits = modrm >> 6;
    let reg_field = (modrm >> 3) & 0x7;
    let rm_idx = modrm & 0x7;

    if mod_bits != 0b11 { return Err(DecodeError::InvalidFormat); }

    let src_reg = reg_from_index(rm_idx);
    let opcode = match reg_field {
        4 => Opcode::MUL,
        7 => Opcode::IDIV,
        _ => return Err(DecodeError::UnknownOpcode(0xF7)),
    };

    Ok(Instruction { opcode, dest: None, src: Some(Operand::Register(src_reg)), length: 2 })
}

/// 0xC1 – SHL (/4), SHR (/5), SAR (/7) with imm8 count
fn decode_c1(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    if bytes.len() < 3 { return Err(DecodeError::InsufficientBytes); }
    let modrm = bytes[1];
    let mod_bits = modrm >> 6;
    let reg_field = (modrm >> 3) & 0x7;
    let rm_idx = modrm & 0x7;

    if mod_bits != 0b11 { return Err(DecodeError::InvalidFormat); }

    let dst_reg = reg_from_index(rm_idx);
    let count = bytes[2] as u32;

    let opcode = match reg_field {
        4 => Opcode::SHL,
        5 => Opcode::SHR,
        7 => Opcode::SAR,
        _ => return Err(DecodeError::UnknownOpcode(0xC1)),
    };

    Ok(Instruction {
        opcode,
        dest: Some(Operand::Register(dst_reg)),
        src: Some(Operand::Immediate(count)),
        length: 3,
    })
}

/// 0x89 (MOV r/m32, r32) and 0x8B (MOV r32, r/m32)
fn decode_mov_rm(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    if bytes.len() < 2 { return Err(DecodeError::InsufficientBytes); }
    let opcode_byte = bytes[0];
    let modrm = bytes[1];
    let mod_bits = modrm >> 6;
    let reg_idx = (modrm >> 3) & 0x7;
    let rm_idx = modrm & 0x7;
    let reg_reg = reg_from_index(reg_idx);

    let after = 2usize; // bytes[2] is first byte after ModRM

    let (rm_operand, extra): (Operand, u8) = match mod_bits {
        0b11 => (Operand::Register(reg_from_index(rm_idx)), 0),
        0b00 => {
            if rm_idx == 4 { return Err(DecodeError::InvalidFormat); } // SIB not supported
            if rm_idx == 5 {
                // [disp32]
                if bytes.len() < after + 4 { return Err(DecodeError::InsufficientBytes); }
                let addr = u32::from_le_bytes([bytes[after], bytes[after+1], bytes[after+2], bytes[after+3]]);
                (Operand::Memory(addr), 4)
            } else {
                // [register]
                let sentinel = 0x8000_0000u32 | rm_idx as u32;
                (Operand::Memory(sentinel), 0)
            }
        }
        0b01 => {
            // [register + disp8]
            if rm_idx == 4 { return Err(DecodeError::InvalidFormat); }
            if bytes.len() < after + 1 { return Err(DecodeError::InsufficientBytes); }
            let disp = bytes[after] as i8 as i32;
            let sentinel = 0x4000_0000u32 | ((rm_idx as u32) << 16) | ((disp as u32) & 0xFFFF);
            (Operand::Memory(sentinel), 1)
        }
        0b10 => {
            // [register + disp32]
            if rm_idx == 4 { return Err(DecodeError::InvalidFormat); }
            if bytes.len() < after + 4 { return Err(DecodeError::InsufficientBytes); }
            let disp = i32::from_le_bytes([bytes[after], bytes[after+1], bytes[after+2], bytes[after+3]]);
            let sentinel = 0x4000_0000u32 | ((rm_idx as u32) << 16) | ((disp as u32) & 0xFFFF);
            (Operand::Memory(sentinel), 4)
        }
        _ => unreachable!(),
    };

    let length = 2 + extra;

    let (dest, src) = if opcode_byte == 0x8B {
        // MOV r32, r/m32  →  dest=reg, src=rm
        (Some(Operand::Register(reg_reg)), Some(rm_operand))
    } else {
        // 0x89 MOV r/m32, r32  →  dest=rm, src=reg
        (Some(rm_operand), Some(Operand::Register(reg_reg)))
    };

    Ok(Instruction { opcode: Opcode::MOV, dest, src, length })
}

// ─── Simple-opcode helpers ────────────────────────────────────────────────────

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
    Ok(reg_from_index(opcode_byte - 0xB8))
}

fn parse_opcode(opcode_byte: u8) -> Result<Opcode, DecodeError> {
    match opcode_byte {
        0x58..=0x5F => Ok(Opcode::POP),
        0x50..=0x57 => Ok(Opcode::PUSH),
        0xE8        => Ok(Opcode::CALL),
        0xB8..=0xBF => Ok(Opcode::MOV),
        0xEB | 0xE9 => Ok(Opcode::JMP),
        0x28..=0x2D => Ok(Opcode::SUB),
        0x00..=0x05 => Ok(Opcode::ADD),
        0x21 | 0x23 => Ok(Opcode::AND),
        0x09 | 0x0B => Ok(Opcode::OR),
        0x99        => Ok(Opcode::CDQ),
        0xC3        => Ok(Opcode::RET),
        _           => Err(DecodeError::UnknownOpcode(opcode_byte)),
    }
}

// ─── Main decode entry point ──────────────────────────────────────────────────

pub fn decode(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    if bytes.is_empty() {
        return Err(DecodeError::InsufficientBytes);
    }

    let opcode_byte = bytes[0];

    // Group / complex encodings handled first
    match opcode_byte {
        0x80 | 0x81 | 0x83 => return decode_group1(bytes),
        0xF7               => return decode_f7(bytes),
        0xC1               => return decode_c1(bytes),
        0x89 | 0x8B        => return decode_mov_rm(bytes),
        _                  => {}
    }

    let opcode = parse_opcode(opcode_byte)?;

    match opcode {
        Opcode::POP => {
            let register = get_pop_register(opcode_byte)?;
            Ok(Instruction { opcode, dest: Some(Operand::Register(register)), src: None, length: 1 })
        }
        Opcode::PUSH => {
            let register = get_push_register(opcode_byte)?;
            Ok(Instruction { opcode, dest: None, src: Some(Operand::Register(register)), length: 1 })
        }
        Opcode::CALL => {
            if bytes.len() < 5 { return Err(DecodeError::InsufficientBytes); }
            let disp = u32::from_le_bytes(bytes[1..5].try_into().unwrap());
            Ok(Instruction { opcode, dest: None, src: Some(Operand::Immediate(disp)), length: 5 })
        }
        Opcode::MOV => {
            if bytes.len() < 5 { return Err(DecodeError::InsufficientBytes); }
            let dest_reg = mov_imm_register(opcode_byte)?;
            let imm = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
            Ok(Instruction {
                opcode,
                dest: Some(Operand::Register(dest_reg)),
                src: Some(Operand::Immediate(imm)),
                length: 5,
            })
        }
        Opcode::JMP => {
            match opcode_byte {
                0xEB => {
                    if bytes.len() < 2 { return Err(DecodeError::InsufficientBytes); }
                    Ok(Instruction {
                        opcode,
                        dest: Some(Operand::Immediate((bytes[1] as i8) as i32 as u32)),
                        src: None,
                        length: 2,
                    })
                }
                0xE9 => {
                    if bytes.len() < 5 { return Err(DecodeError::InsufficientBytes); }
                    let displacement = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                    Ok(Instruction {
                        opcode,
                        dest: Some(Operand::Immediate(displacement)),
                        src: None,
                        length: 5,
                    })
                }
                _ => Err(DecodeError::UnknownOpcode(opcode_byte)),
            }
        }
        Opcode::SUB => {
            if bytes.len() < 2 { return Err(DecodeError::InsufficientBytes); }
            let modrm = bytes[1];
            let mod_bits = modrm >> 6;
            if mod_bits != 0b11 { return Err(DecodeError::InvalidFormat); }
            // 0x29 / 0x28: reg=src, rm=dst; 0x2B / 0x2A: reg=dst, rm=src
            let (dest_reg, src_reg) = if opcode_byte == 0x29 || opcode_byte == 0x28 {
                (reg_from_index(modrm & 0x7), reg_from_index((modrm >> 3) & 0x7))
            } else {
                (reg_from_index((modrm >> 3) & 0x7), reg_from_index(modrm & 0x7))
            };
            Ok(Instruction {
                opcode,
                dest: Some(Operand::Register(dest_reg)),
                src: Some(Operand::Register(src_reg)),
                length: 2,
            })
        }
        Opcode::ADD => {
            if bytes.len() < 2 { return Err(DecodeError::InsufficientBytes); }
            let modrm = bytes[1];
            let mod_bits = modrm >> 6;
            if mod_bits != 0b11 { return Err(DecodeError::InvalidFormat); }
            // 0x01 / 0x00: reg=src, rm=dst; 0x03 / 0x02: reg=dst, rm=src
            let (dest_reg, src_reg) = if opcode_byte == 0x01 || opcode_byte == 0x00 {
                (reg_from_index(modrm & 0x7), reg_from_index((modrm >> 3) & 0x7))
            } else {
                (reg_from_index((modrm >> 3) & 0x7), reg_from_index(modrm & 0x7))
            };
            Ok(Instruction {
                opcode,
                dest: Some(Operand::Register(dest_reg)),
                src: Some(Operand::Register(src_reg)),
                length: 2,
            })
        }
        Opcode::AND => {
            if bytes.len() < 2 { return Err(DecodeError::InsufficientBytes); }
            let modrm = bytes[1];
            if modrm >> 6 != 0b11 { return Err(DecodeError::InvalidFormat); }
            let (dest_reg, src_reg) = if opcode_byte == 0x21 {
                (reg_from_index(modrm & 0x7), reg_from_index((modrm >> 3) & 0x7))
            } else {
                (reg_from_index((modrm >> 3) & 0x7), reg_from_index(modrm & 0x7))
            };
            Ok(Instruction {
                opcode,
                dest: Some(Operand::Register(dest_reg)),
                src: Some(Operand::Register(src_reg)),
                length: 2,
            })
        }
        Opcode::OR => {
            if bytes.len() < 2 { return Err(DecodeError::InsufficientBytes); }
            let modrm = bytes[1];
            if modrm >> 6 != 0b11 { return Err(DecodeError::InvalidFormat); }
            let (dest_reg, src_reg) = if opcode_byte == 0x09 {
                (reg_from_index(modrm & 0x7), reg_from_index((modrm >> 3) & 0x7))
            } else {
                (reg_from_index((modrm >> 3) & 0x7), reg_from_index(modrm & 0x7))
            };
            Ok(Instruction {
                opcode,
                dest: Some(Operand::Register(dest_reg)),
                src: Some(Operand::Register(src_reg)),
                length: 2,
            })
        }
        Opcode::CDQ => Ok(Instruction { opcode, dest: None, src: None, length: 1 }),
        Opcode::RET => Ok(Instruction { opcode, dest: None, src: None, length: 1 }),
        Opcode::MUL | Opcode::IDIV | Opcode::SHL | Opcode::SHR | Opcode::SAR => {
            Err(DecodeError::UnknownOpcode(opcode_byte))
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::RegisterName;

    #[test]
    fn test_parse_opcode_push() {
        assert_eq!(parse_opcode(0x50), Ok(Opcode::PUSH));
        assert_eq!(parse_opcode(0x57), Ok(Opcode::PUSH));
    }

    #[test]
    fn test_parse_opcode_unknown() {
        assert!(parse_opcode(0x06).is_err());
        assert!(parse_opcode(0x10).is_err());
    }

    #[test]
    fn test_get_push_register() {
        assert_eq!(get_push_register(0x50), Ok(RegisterName::EAX));
        assert_eq!(get_push_register(0x53), Ok(RegisterName::EBX));
    }

    #[test]
    fn test_decode_push_eax() {
        let bytes = [0x50];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::PUSH);
        assert_eq!(instr.src, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instr.length, 1);
    }

    #[test]
    fn test_decode_empty_bytes() {
        assert!(decode(&[]).is_err());
    }

    #[test]
    fn test_decode_mov_imm_to_eax() {
        let bytes = [0xB8, 0xEF, 0xBE, 0xAD, 0xDE];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::MOV);
        assert_eq!(instr.dest, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instr.src, Some(Operand::Immediate(0xDEADBEEF)));
        assert_eq!(instr.length, 5);
    }

    #[test]
    fn test_decode_mul_ebx() {
        // MUL EBX: 0xF7 /4 EBX → 0xF7 0xE3 (mod=11, reg=4, rm=3)
        let bytes = [0xF7, 0xE3];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::MUL);
        assert_eq!(instr.src, Some(Operand::Register(RegisterName::EBX)));
    }

    #[test]
    fn test_decode_idiv_ecx() {
        // IDIV ECX: 0xF7 /7 ECX → 0xF7 0xF9 (mod=11, reg=7, rm=1)
        let bytes = [0xF7, 0xF9];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::IDIV);
        assert_eq!(instr.src, Some(Operand::Register(RegisterName::ECX)));
    }

    #[test]
    fn test_decode_shl_eax_4() {
        // SHL EAX, 4: 0xC1 0xE0 0x04 (mod=11, reg=4, rm=0)
        let bytes = [0xC1, 0xE0, 0x04];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::SHL);
        assert_eq!(instr.dest, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instr.src, Some(Operand::Immediate(4)));
    }

    #[test]
    fn test_decode_sar_edx_16() {
        // SAR EDX, 16: 0xC1 0xFA 0x10 (mod=11, reg=7, rm=2)
        let bytes = [0xC1, 0xFA, 0x10];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::SAR);
        assert_eq!(instr.dest, Some(Operand::Register(RegisterName::EDX)));
        assert_eq!(instr.src, Some(Operand::Immediate(16)));
    }

    #[test]
    fn test_decode_and_reg_reg() {
        // AND EAX, EBX: 0x21 (reg=EBX=3, rm=EAX=0) → 0x21 0xD8
        let bytes = [0x21, 0xD8];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::AND);
        assert_eq!(instr.dest, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instr.src, Some(Operand::Register(RegisterName::EBX)));
    }

    #[test]
    fn test_decode_and_imm() {
        // AND EAX, 0xFF: 0x81 0xE0 0xFF 0x00 0x00 0x00
        let bytes = [0x81, 0xE0, 0xFF, 0x00, 0x00, 0x00];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::AND);
        assert_eq!(instr.dest, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instr.src, Some(Operand::Immediate(0xFF)));
    }

    #[test]
    fn test_decode_or_reg_reg() {
        // OR EAX, ECX: 0x09 0xC8 (reg=ECX=1, rm=EAX=0)
        let bytes = [0x09, 0xC8];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::OR);
        assert_eq!(instr.dest, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instr.src, Some(Operand::Register(RegisterName::ECX)));
    }

    #[test]
    fn test_decode_cdq() {
        let instr = decode(&[0x99]).unwrap();
        assert_eq!(instr.opcode, Opcode::CDQ);
        assert_eq!(instr.length, 1);
    }

    #[test]
    fn test_decode_mov_mem_absolute() {
        // MOV EAX, [0x2000]: 0x8B 0x05 0x00 0x20 0x00 0x00
        let bytes = [0x8B, 0x05, 0x00, 0x20, 0x00, 0x00];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::MOV);
        assert_eq!(instr.dest, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instr.src, Some(Operand::Memory(0x2000)));
        assert_eq!(instr.length, 6);
    }

    #[test]
    fn test_decode_mov_store_absolute() {
        // MOV [0x2008], EAX: 0x89 0x05 0x08 0x20 0x00 0x00
        let bytes = [0x89, 0x05, 0x08, 0x20, 0x00, 0x00];
        let instr = decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::MOV);
        assert_eq!(instr.dest, Some(Operand::Memory(0x2008)));
        assert_eq!(instr.src, Some(Operand::Register(RegisterName::EAX)));
        assert_eq!(instr.length, 6);
    }
}
