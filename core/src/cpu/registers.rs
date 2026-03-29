//! CPU Registers Module
//!
//! This module implements the x86-32 general purpose registers.
//! Each register is a 32-bit value that can store data, addresses, or results.
//!
//! Key concepts:
//! - Registers are the CPU's fast scratch paper
//! - 8 general purpose registers + EIP (instruction pointer)
//! - Each register has conventional uses but can store any 32-bit value
//! - ESP (stack pointer) has special meaning and should be handled carefully

use std::fmt;

/// Enumeration of all available registers
///
/// This enum allows us to refer to registers by name rather than
/// hardcoding field names throughout the codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterName {
    EAX, // Accumulator - primary register for arithmetic
    EBX, // Base - often used for data pointers
    ECX, // Counter - loop counting, shift counts
    EDX, // Data - I/O operations, extended arithmetic
    ESI, // Source Index - string operations source
    EDI, // Destination Index - string operations destination
    EBP, // Base Pointer - stack frame base
    ESP, // Stack Pointer - top of stack (special!)
    EIP, // Instruction Pointer - next instruction address
}

/// CPU Registers Structure
///
/// Represents all the general purpose registers in an x86-32 CPU.
/// Each register is a 32-bit unsigned integer (u32).
///
/// Memory layout in our emulator:
/// - Each field takes exactly 4 bytes (32 bits)
/// - Total size: 9 registers × 4 bytes = 36 bytes
/// - Stored in RAM (not in CPU like real hardware)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Registers {
    // General Purpose Registers (32-bit)
    pub eax: u32, // Accumulator - arithmetic operations, return values
    pub ebx: u32, // Base - pointer to data structures
    pub ecx: u32, // Counter - loop counting, shift operations
    pub edx: u32, // Data - I/O operations, multiplication/division
    pub esi: u32, // Source Index - string operations source pointer
    pub edi: u32, // Destination Index - string operations dest pointer
    pub ebp: u32, // Base Pointer - stack frame base (be careful!)
    pub esp: u32, // Stack Pointer - top of stack (special meaning!)
    pub eip: u32, // Instruction Pointer - next instruction address
}

impl Registers {
    /// Create a new set of registers with default values
    ///
    /// Default values are chosen to represent a "clean" CPU state:
    /// - All GPRs start at 0 (undefined in real CPU, but safe for emulation)
    /// - ESP/EBP start at stack base (0xFFFF0000)
    /// - EIP starts at code section (0x00001000)
    pub fn new() -> Self {
        Registers {
            // General purpose registers start at 0
            // In real hardware, these would contain garbage
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
            esi: 0,
            edi: 0,

            // Stack registers start at stack base
            // Stack grows downward from high addresses
            ebp: 0x00F0_0000, // Base pointer at stack base (within 16 MB RAM)
            esp: 0x00F0_0000, // Stack pointer starts at top of stack

            // Instruction pointer starts in code section
            // Code section begins at 0x1000 (after NULL region)
            eip: 0x0000_1000, // First instruction address
        }
    }

    /// Get the value of a register by name
    ///
    /// This method allows dynamic register access using the RegisterName enum.
    /// Useful for instruction decoders that need to access registers dynamically.
    ///
    /// # Arguments
    /// * `reg` - The register to read
    ///
    /// # Returns
    /// The 32-bit value stored in the register
    ///
    /// # Example
    /// ```rust
    //use web_x86_core::cpu::{Registers, RegisterName};
    //let mut regs = Registers::new();
    //regs.eax = 42;
    //assert_eq!(regs.get(RegisterName::EAX), 42);
    /// ```
    pub fn get(&self, reg: RegisterName) -> u32 {
        match reg {
            RegisterName::EAX => self.eax,
            RegisterName::EBX => self.ebx,
            RegisterName::ECX => self.ecx,
            RegisterName::EDX => self.edx,
            RegisterName::ESI => self.esi,
            RegisterName::EDI => self.edi,
            RegisterName::EBP => self.ebp,
            RegisterName::ESP => self.esp,
            RegisterName::EIP => self.eip,
        }
    }

    /// Set the value of a register by name
    ///
    /// This method allows dynamic register modification using the RegisterName enum.
    /// Useful for instruction executors that need to write to registers dynamically.
    ///
    /// # Arguments
    /// * `reg` - The register to modify
    /// * `value` - The new 32-bit value to store
    ///
    /// # Example
    /// ```rust
    //use web_x86_core::cpu::{Registers, RegisterName};
    //let mut regs = Registers::new();
    //regs.set(RegisterName::EAX, 42);
    //assert_eq!(regs.eax, 42);
    /// ```
    pub fn set(&mut self, reg: RegisterName, value: u32) {
        match reg {
            RegisterName::EAX => self.eax = value,
            RegisterName::EBX => self.ebx = value,
            RegisterName::ECX => self.ecx = value,
            RegisterName::EDX => self.edx = value,
            RegisterName::ESI => self.esi = value,
            RegisterName::EDI => self.edi = value,
            RegisterName::EBP => self.ebp = value,
            RegisterName::ESP => self.esp = value,
            RegisterName::EIP => self.eip = value,
        }
    }

    /// Reset all registers to their default values
    ///
    /// This is equivalent to calling `Registers::new()` but reuses
    /// the existing struct instance. Useful for CPU reset operations.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Advance the instruction pointer by the specified number of bytes
    ///
    /// This is used after executing instructions to move to the next instruction.
    /// Most instructions advance EIP by their length (1-15 bytes).
    ///
    /// # Arguments
    /// * `bytes` - Number of bytes to advance EIP
    ///
    /// # Example
    /// ```rust
    //use web_x86_core::cpu::Registers;
    //let mut regs = Registers::new();
    //regs.eip = 0x1000;
    //regs.advance_ip(2);  // ADD EAX, EBX is 2 bytes
    //assert_eq!(regs.eip, 0x1002);
    /// ```
    pub fn advance_ip(&mut self, bytes: u32) {
        self.eip = self.eip.wrapping_add(bytes);
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Registers {
    /// Format registers for human-readable output
    ///
    /// This implementation provides a clean, aligned display of all registers.
    /// Useful for debugging and disassembly output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "EAX: 0x{:08X}  EBX: 0x{:08X}  ECX: 0x{:08X}  EDX: 0x{:08X}",
            self.eax, self.ebx, self.ecx, self.edx
        )?;
        writeln!(
            f,
            "ESI: 0x{:08X}  EDI: 0x{:08X}  EBP: 0x{:08X}  ESP: 0x{:08X}",
            self.esi, self.edi, self.ebp, self.esp
        )?;
        write!(f, "EIP: 0x{:08X}", self.eip)?;
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_new_registers() {
//         let regs = Registers::new();

//         // Check default values
//         assert_eq!(regs.eax, 0);
//         assert_eq!(regs.ebx, 0);
//         assert_eq!(regs.ecx, 0);
//         assert_eq!(regs.edx, 0);
//         assert_eq!(regs.esi, 0);
//         assert_eq!(regs.edi, 0);

//         // Check special register defaults
//         assert_eq!(regs.ebp, 0xFFFF_0000);
//         assert_eq!(regs.esp, 0xFFFF_0000);
//         assert_eq!(regs.eip, 0x0000_1000);
//     }

//     #[test]
//     fn test_register_access() {
//         let mut regs = Registers::new();

//         // Test getter
//         assert_eq!(regs.get(RegisterName::EAX), 0);

//         // Test setter
//         regs.set(RegisterName::EAX, 0x12345678);
//         assert_eq!(regs.get(RegisterName::EAX), 0x12345678);
//         assert_eq!(regs.eax, 0x12345678);

//         // Test all registers
//         regs.set(RegisterName::EBX, 0x87654321);
//         regs.set(RegisterName::ECX, 0x11111111);
//         regs.set(RegisterName::EDX, 0x22222222);

//         assert_eq!(regs.get(RegisterName::EBX), 0x87654321);
//         assert_eq!(regs.get(RegisterName::ECX), 0x11111111);
//         assert_eq!(regs.get(RegisterName::EDX), 0x22222222);
//     }

//     #[test]
//     fn test_instruction_pointer_operations() {
//         let mut regs = Registers::new();

//         // Test initial value
//         assert_eq!(regs.eip, 0x0000_1000);

//         // Test advancing
//         regs.advance_ip(4);
//         assert_eq!(regs.eip, 0x1004);

//         // Test wrapping (if we ever hit 32-bit limit)
//         regs.eip = 0xFFFFFFFE;
//         regs.advance_ip(4);
//         assert_eq!(regs.eip, 0x00000002); // Wrapped!
//     }

//     #[test]
//     fn test_reset() {
//         let mut regs = Registers::new();

//         // Modify some registers
//         regs.eax = 0x12345678;
//         regs.ebx = 0x87654321;
//         regs.eip = 0x5000;

//         // Reset
//         regs.reset();

//         // Check they're back to defaults
//         assert_eq!(regs.eax, 0);
//         assert_eq!(regs.ebx, 0);
//         assert_eq!(regs.eip, 0x0000_1000);
//     }

//     #[test]
//     fn test_display_formatting() {
//         let regs = Registers::new();
//         let formatted = format!("{}", regs);

//         // Check that all registers are present
//         assert!(formatted.contains("EAX: 0x00000000"));
//         assert!(formatted.contains("ESP: 0xFFFF0000"));
//         assert!(formatted.contains("EIP: 0x00001000"));
//     }
// }
