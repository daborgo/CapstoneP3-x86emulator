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
}

