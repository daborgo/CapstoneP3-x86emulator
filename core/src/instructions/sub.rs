// sub.rs
// x86 SUB: dest = dest - src; updates CF, OF, ZF, SF, PF, AF.

use crate::cpu::{CPU};
use crate::memory::Memory;
use crate::decoder::{Instruction, Operand};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidOperand,
    MemoryAccessError,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidOperand => write!(f, "Invalid operand for SUB instruction"),
            ExecutionError::MemoryAccessError => write!(f, "Memory access error during SUB"),
        }
    }
}

impl std::error::Error for ExecutionError {}

/// Execute a SUB instruction
pub fn execute(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    // Get destination value (register or memory)
    let (dest_val, dest_addr, dest_reg) = match &instruction.dest {
        Some(Operand::Register(reg)) => (cpu.registers.get(*reg), None, Some(*reg)),
        Some(Operand::Memory(addr)) => (memory.read_u32(*addr).map_err(|_| ExecutionError::MemoryAccessError)?, Some(*addr), None),
        _ => return Err(ExecutionError::InvalidOperand),
    };

    // Get source value (register, immediate, or memory)
    let src_val = match &instruction.src {
        Some(Operand::Register(reg)) => cpu.registers.get(*reg),
        Some(Operand::Immediate(imm)) => *imm,
        Some(Operand::Memory(addr)) => memory.read_u32(*addr).map_err(|_| ExecutionError::MemoryAccessError)?,
        _ => return Err(ExecutionError::InvalidOperand),
    };

    let (result, flags) = sub_core(CpuFlags {
        cf: cpu.flags.cf,
        pf: cpu.flags.pf,
        af: cpu.flags.af,
        zf: cpu.flags.zf,
        sf: cpu.flags.sf,
        of: cpu.flags.of,
    }, dest_val, src_val, 32);

    // Write result back to destination
    if let Some(reg) = dest_reg {
        cpu.registers.set(reg, result);
    } else if let Some(addr) = dest_addr {
        memory.write_u32(addr, result).map_err(|_| ExecutionError::MemoryAccessError)?;
    }

    // Update CPU flags
    cpu.flags.cf = flags.cf;
    cpu.flags.pf = flags.pf;
    cpu.flags.af = flags.af;
    cpu.flags.zf = flags.zf;
    cpu.flags.sf = flags.sf;
    cpu.flags.of = flags.of;

    // Advance instruction pointer
    cpu.registers.advance_ip(instruction.length as u32);

    Ok(())
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CpuFlags {
    pub cf: bool, // Carry (borrow for SUB)
    pub pf: bool, // Parity (even parity of low byte)
    pub af: bool, // Adjust (borrow/carry out of bit 3)
    pub zf: bool, // Zero
    pub sf: bool, // Sign
    pub of: bool, // Overflow
}

impl CpuFlags {
    #[inline]
    fn set_szp_u32(&mut self, res: u32, width_bits: u32) {
        let mask = if width_bits == 8 { 0xFF } else if width_bits == 16 { 0xFFFF } else { 0xFFFF_FFFF };
        let v = res & mask;
        self.zf = v == 0;
        self.sf = ((v >> (width_bits - 1)) & 1) != 0;
        self.pf = even_parity8(v as u8);
    }
}

/// Even parity (true if the number of set bits in the low 8 bits is even)
#[inline]
fn even_parity8(x: u8) -> bool {
    (x.count_ones() & 1) == 0
}

/// Compute OF for subtraction: dest - src (both masked to width)
/// OF = ((dest ^ src) & (dest ^ res) & sign_bit) != 0
#[inline]
fn overflow_sub(dest: u32, src: u32, res: u32, sign_bit: u32) -> bool {
    (((dest ^ src) & (dest ^ res)) & sign_bit) != 0
}

/// Adjust flag (AF) for add/sub can be computed with XOR trick on bit 4.
/// For SUB it’s the same XOR relation:
/// AF = ((dest ^ src ^ res) & 0x10) != 0
#[inline]
fn adjust_flag(dest: u32, src: u32, res: u32) -> bool {
    ((dest ^ src ^ res) & 0x10) != 0
}

/// Core SUB for arbitrary width (8/16/32). Returns (result, updated flags).
#[inline]
fn sub_core(mut flags: CpuFlags, dest: u32, src: u32, width_bits: u32) -> (u32, CpuFlags) {
    let (mask, sign_bit) = match width_bits {
        8  => (0xFFu32, 0x80u32),
        16 => (0xFFFFu32, 0x8000u32),
        32 => (0xFFFF_FFFFu32, 0x8000_0000u32),
        _  => unreachable!("width_bits must be 8/16/32"),
    };

    let d = dest & mask;
    let s = src  & mask;
    let res = d.wrapping_sub(s) & mask;

    // CF: borrow occurred on unsigned subtraction
    flags.cf = d < s;

    // OF: signed overflow
    flags.of = overflow_sub(d, s, res, sign_bit);

    // AF: borrow/carry out of bit 3 (low nibble)
    flags.af = adjust_flag(d, s, res);

    // ZF, SF, PF
    flags.set_szp_u32(res, width_bits);

    (res, flags)
}

// Public entry points

#[inline]
pub fn sub8(flags: CpuFlags, dest: u8, src: u8) -> (u8, CpuFlags) {
    let (r, f) = sub_core(flags, dest as u32, src as u32, 8);
    (r as u8, f)
}

#[inline]
pub fn sub16(flags: CpuFlags, dest: u16, src: u16) -> (u16, CpuFlags) {
    let (r, f) = sub_core(flags, dest as u32, src as u32, 16);
    (r as u16, f)
}

#[inline]
pub fn sub32(flags: CpuFlags, dest: u32, src: u32) -> (u32, CpuFlags) {
    sub_core(flags, dest, src, 32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::{Instruction, Operand};
    use crate::cpu::RegisterName;

    #[test]
    fn test_sub_reg_reg() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x2000);
        cpu.registers.eax = 10;
        cpu.registers.ebx = 3;
        let instruction = Instruction {
            opcode: crate::decoder::Opcode::SUB,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Register(RegisterName::EBX)),
            length: 1,
        };
        super::execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(cpu.registers.eax, 7);
    }

    #[test]
    fn test_sub_reg_mem() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x2000);
        cpu.registers.eax = 20;
        memory.write_u32(0x1100, 5).unwrap();
        let instruction = Instruction {
            opcode: crate::decoder::Opcode::SUB,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Memory(0x1100)),
            length: 1,
        };
        super::execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(cpu.registers.eax, 15);
    }

    #[test]
    fn test_sub_mem_reg() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x2000);
        memory.write_u32(0x1200, 50).unwrap();
        cpu.registers.ebx = 8;
        let instruction = Instruction {
            opcode: crate::decoder::Opcode::SUB,
            dest: Some(Operand::Memory(0x1200)),
            src: Some(Operand::Register(RegisterName::EBX)),
            length: 1,
        };
        super::execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(memory.read_u32(0x1200).unwrap(), 42);
    }

    #[test]
    fn test_sub_mem_mem() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x2000);
        memory.write_u32(0x1300, 100).unwrap();
        memory.write_u32(0x1400, 25).unwrap();
        let instruction = Instruction {
            opcode: crate::decoder::Opcode::SUB,
            dest: Some(Operand::Memory(0x1300)),
            src: Some(Operand::Memory(0x1400)),
            length: 1,
        };
        super::execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(memory.read_u32(0x1300).unwrap(), 75);
    }

    #[test]
    fn test_sub_reg_imm() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x2000);
        cpu.registers.eax = 30;
        let instruction = Instruction {
            opcode: crate::decoder::Opcode::SUB,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Immediate(10)),
            length: 1,
        };
        super::execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(cpu.registers.eax, 20);
    }

    #[test]
    fn test_sub_mem_imm() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x2000);
        memory.write_u32(0x1500, 60).unwrap();
        let instruction = Instruction {
            opcode: crate::decoder::Opcode::SUB,
            dest: Some(Operand::Memory(0x1500)),
            src: Some(Operand::Immediate(15)),
            length: 1,
        };
        super::execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(memory.read_u32(0x1500).unwrap(), 45);
    }
}

