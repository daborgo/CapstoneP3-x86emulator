//! CPU Module
//! 
//! This module contains all CPU-related functionality including
//! registers, flags, and instruction execution.

pub mod registers;
pub mod flags;

// Re-export main types for easier access
pub use registers::{Registers, RegisterName};
pub use flags::Flags;

/// Main CPU structure combining all components
/// 
/// The CPU owns the registers and flags, and works with memory
/// to execute instructions.
#[derive(Debug, Clone)]
pub struct CPU {
    /// General purpose registers (EAX, EBX, etc.)
    pub registers: Registers,
    
    /// Status flags (CF, ZF, SF, etc.)
    pub flags: Flags,
}

impl CPU {
    /// Create a new CPU with default register and flag values
    pub fn new() -> Self {
        CPU {
            registers: Registers::new(),
            flags: Flags::new(),
        }
    }
    
    /// Reset the CPU to initial state
    pub fn reset(&mut self) {
        self.registers.reset();
        self.flags.clear_all();
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_creation() {
        let cpu = CPU::new();
        
        // Verify registers are initialized
        assert_eq!(cpu.registers.eax, 0);
        assert_eq!(cpu.registers.eip, 0x1000);
        
        // Verify flags are cleared
        assert!(!cpu.flags.cf);
        assert!(!cpu.flags.zf);
    }
    
    #[test]
    fn test_cpu_reset() {
        let mut cpu = CPU::new();
        
        // Modify state
        cpu.registers.eax = 0x12345678;
        cpu.flags.zf = true;
        
        // Reset
        cpu.reset();
        
        // Verify back to defaults
        assert_eq!(cpu.registers.eax, 0);
        assert!(!cpu.flags.zf);
    }

    #[test]
    fn test_register_set_get() {
        let mut cpu = CPU::new();
        cpu.registers.eax = 0xDEADBEEF;
        assert_eq!(cpu.registers.eax, 0xDEADBEEF);
        cpu.registers.ebx = 0x12345678;
        assert_eq!(cpu.registers.ebx, 0x12345678);
    }

    #[test]
    fn test_flag_manipulation() {
        let mut cpu = CPU::new();
        // Set and clear Carry Flag
        cpu.flags.cf = true;
        assert!(cpu.flags.cf);
        cpu.flags.cf = false;
        assert!(!cpu.flags.cf);
        // Set and clear Zero Flag
        cpu.flags.zf = true;
        assert!(cpu.flags.zf);
        cpu.flags.zf = false;
        assert!(!cpu.flags.zf);
    }

    #[test]
    fn test_register_overflow() {
        let mut cpu = CPU::new();
        cpu.registers.eax = u32::MAX;
        cpu.registers.eax = cpu.registers.eax.wrapping_add(1);
        assert_eq!(cpu.registers.eax, 0);
    }

    #[test]
    fn test_combined_register_flag_operation() {
        let mut cpu = CPU::new();
        cpu.registers.eax = 1;
        cpu.registers.ebx = 2;
        // Simulate an ADD instruction manually
        let result = cpu.registers.eax.wrapping_add(cpu.registers.ebx);
        cpu.registers.eax = result;
        // Manually update Zero and Carry flags for this test
        cpu.flags.zf = cpu.registers.eax == 0;
        cpu.flags.cf = (cpu.registers.eax as u64) < (1u64 + 2u64); // No carry expected
        assert_eq!(cpu.registers.eax, 3);
        assert!(!cpu.flags.zf);
        assert!(!cpu.flags.cf);
    }
}

