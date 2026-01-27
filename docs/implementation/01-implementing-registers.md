# Implementing the Registers Module

## Overview
This document provides a step-by-step guide to implementing the CPU registers in Rust. Every line of code is explained with the underlying concepts and reasoning.

## File: `core/src/cpu/registers.rs`

### Step 1: Module Declaration and Imports

```rust
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
```

**Explanation**:
- `//!` creates module-level documentation (shows up in `cargo doc`)
- Comments explain the purpose and key concepts
- `use std::fmt;` for implementing `Display` trait later

### Step 2: Register Name Enumeration

```rust
/// Enumeration of all available registers
/// 
/// This enum allows us to refer to registers by name rather than
/// hardcoding field names throughout the codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterName {
    EAX,  // Accumulator - primary register for arithmetic
    EBX,  // Base - often used for data pointers
    ECX,  // Counter - loop counting, shift counts
    EDX,  // Data - I/O operations, extended arithmetic
    ESI,  // Source Index - string operations source
    EDI,  // Destination Index - string operations destination
    EBP,  // Base Pointer - stack frame base
    ESP,  // Stack Pointer - top of stack (special!)
    EIP,  // Instruction Pointer - next instruction address
}
```

**Why an enum?**
- **Type safety**: Can't accidentally use wrong register name
- **Maintainability**: Change register name in one place
- **Clarity**: `RegisterName::EAX` is clearer than string "EAX"
- **Pattern matching**: Can use `match` statements for register-specific logic

### Step 3: Registers Struct Definition

```rust
/// CPU Registers Structure
/// 
/// Represents all the general purpose registers in an x86-32 CPU.
/// Each register is a 32-bit unsigned integer (u32).
/// 
/// Memory layout in our emulator:
/// - Each field takes exactly 4 bytes (32 bits)
/// - Total size: 9 registers × 4 bytes = 36 bytes
/// - Stored in RAM (not in CPU like real hardware)
pub struct Registers {
    // General Purpose Registers (32-bit)
    pub eax: u32,  // Accumulator - arithmetic operations, return values
    pub ebx: u32,  // Base - pointer to data structures
    pub ecx: u32,  // Counter - loop counting, shift operations
    pub edx: u32,  // Data - I/O operations, multiplication/division
    pub esi: u32,  // Source Index - string operations source pointer
    pub edi: u32,  // Destination Index - string operations dest pointer
    pub ebp: u32,  // Base Pointer - stack frame base (be careful!)
    pub esp: u32,  // Stack Pointer - top of stack (special meaning!)
    pub eip: u32,  // Instruction Pointer - next instruction address
}
```

**Design decisions explained**:

1. **`pub` fields**: Made public for simplicity. Later we might add getters/setters for validation
2. **`u32` type**: Exactly 32 bits, matches x86-32 architecture
3. **Field order**: Grouped logically (GPRs first, then special registers)
4. **Comments**: Each field explains its conventional use

**Why u32 instead of i32?**
- Registers don't have inherent signedness
- Signed/unsigned interpretation depends on the instruction
- `u32` makes bit operations clearer
- Can cast to `i32` when needed: `eax as i32`

### Step 4: Constructor Implementation

```rust
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
            ebp: 0xFFFF_0000,  // Base pointer at stack base
            esp: 0xFFFF_0000,  // Stack pointer starts at top of stack
            
            // Instruction pointer starts in code section
            // Code section begins at 0x1000 (after NULL region)
            eip: 0x0000_1000,  // First instruction address
        }
    }
}
```

**Why these default values?**

1. **GPRs = 0**: Safe default, no undefined behavior
2. **ESP/EBP = 0xFFFF0000**: 
   - Stack grows downward (high to low addresses)
   - 0xFFFF0000 is near the top of 32-bit address space
   - Leaves room for stack growth
3. **EIP = 0x1000**: 
   - Avoids NULL region (0x0000-0x0FFF)
   - Code section starts at 0x1000
   - Matches our memory layout

### Step 5: Register Access Methods

```rust
impl Registers {
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
    /// let mut regs = Registers::new();
    /// regs.eax = 42;
    /// assert_eq!(regs.get(RegisterName::EAX), 42);
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
    /// let mut regs = Registers::new();
    /// regs.set(RegisterName::EAX, 42);
    /// assert_eq!(regs.eax, 42);
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
}
```

**Why getter/setter methods?**

1. **Dynamic access**: Instruction decoder can access registers by name
2. **Future extensibility**: Can add validation, logging, or debugging
3. **Consistency**: Same interface for all register access
4. **Safety**: Can't accidentally access wrong register

**Pattern matching explanation**:
- `match` is exhaustive - compiler ensures we handle all cases
- Each arm maps enum variant to struct field
- `&self` vs `&mut self` - read vs write access

### Step 6: Helper Methods

```rust
impl Registers {
    /// Reset all registers to their default values
    /// 
    /// This is equivalent to calling `Registers::new()` but reuses
    /// the existing struct instance. Useful for CPU reset operations.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    /// Get the current stack pointer value
    /// 
    /// ESP is special because it must always point to valid stack memory.
    /// This method provides a safe way to access it with documentation.
    /// 
    /// # Returns
    /// The current stack pointer address
    /// 
    /// # Safety
    /// The returned address should be checked for validity before use
    pub fn stack_pointer(&self) -> u32 {
        self.esp
    }
    
    /// Set the stack pointer to a new value
    /// 
    /// # Arguments
    /// * `value` - The new stack pointer address
    /// 
    /// # Safety
    /// The caller must ensure the address points to valid stack memory
    /// 
    /// # Warning
    /// Incorrect ESP values can cause stack corruption!
    pub fn set_stack_pointer(&mut self, value: u32) {
        self.esp = value;
    }
    
    /// Get the current instruction pointer value
    /// 
    /// EIP points to the next instruction to execute.
    /// This method provides safe access with documentation.
    /// 
    /// # Returns
    /// The address of the next instruction
    pub fn instruction_pointer(&self) -> u32 {
        self.eip
    }
    
    /// Set the instruction pointer to a new value
    /// 
    /// This is used by jump instructions (JMP, CALL, RET).
    /// 
    /// # Arguments
    /// * `value` - The address of the next instruction to execute
    /// 
    /// # Safety
    /// The caller must ensure the address points to valid code
    pub fn set_instruction_pointer(&mut self, value: u32) {
        self.eip = value;
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
    /// let mut regs = Registers::new();
    /// regs.eip = 0x1000;
    /// regs.advance_ip(2);  // ADD EAX, EBX is 2 bytes
    /// assert_eq!(regs.eip, 0x1002);
    /// ```
    pub fn advance_ip(&mut self, bytes: u32) {
        self.eip = self.eip.wrapping_add(bytes);
    }
    
    /// Check if a register is the stack pointer
    /// 
    /// ESP has special meaning and requires careful handling.
    /// This method helps identify when ESP is being modified.
    /// 
    /// # Arguments
    /// * `reg` - The register to check
    /// 
    /// # Returns
    /// True if the register is ESP
    pub fn is_stack_pointer(&self, reg: RegisterName) -> bool {
        reg == RegisterName::ESP
    }
    
    /// Check if a register is the instruction pointer
    /// 
    /// EIP modification requires special handling (jumps, calls, returns).
    /// This method helps identify when EIP is being modified.
    /// 
    /// # Arguments
    /// * `reg` - The register to check
    /// 
    /// # Returns
    /// True if the register is EIP
    pub fn is_instruction_pointer(&self, reg: RegisterName) -> bool {
        reg == RegisterName::EIP
    }
}
```

**Helper method explanations**:

1. **`reset()`**: Convenient way to reset CPU state
2. **`stack_pointer()`/`set_stack_pointer()`**: Safe ESP access with warnings
3. **`instruction_pointer()`/`set_instruction_pointer()`**: Safe EIP access
4. **`advance_ip()`**: Common operation after instruction execution
5. **`is_stack_pointer()`/`is_instruction_pointer()`**: Type checking for special registers

### Step 7: Display Implementation

```rust
impl fmt::Display for Registers {
    /// Format registers for human-readable output
    /// 
    /// This implementation provides a clean, aligned display of all registers.
    /// Useful for debugging and disassembly output.
    /// 
    /// # Example
    /// ```
    /// let regs = Registers::new();
    /// println!("{}", regs);
    /// // Output:
    /// // EAX: 0x00000000  EBX: 0x00000000  ECX: 0x00000000  EDX: 0x00000000
    /// // ESI: 0x00000000  EDI: 0x00000000  EBP: 0xFFFF0000  ESP: 0xFFFF0000
    /// // EIP: 0x00001000
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "EAX: 0x{:08X}  EBX: 0x{:08X}  ECX: 0x{:08X}  EDX: 0x{:08X}", 
                 self.eax, self.ebx, self.ecx, self.edx)?;
        writeln!(f, "ESI: 0x{:08X}  EDI: 0x{:08X}  EBP: 0x{:08X}  ESP: 0x{:08X}", 
                 self.esi, self.edi, self.ebp, self.esp)?;
        writeln!(f, "EIP: 0x{:08X}", self.eip)?;
        Ok(())
    }
}
```

**Display implementation details**:

1. **`{:08X}`**: Format as uppercase hex with 8 digits (32 bits)
2. **`writeln!`**: Write with newline, returns `fmt::Result`
3. **`?` operator**: Propagate errors up the call stack
4. **Alignment**: Registers grouped logically for readability

### Step 8: Debug Implementation

```rust
impl fmt::Debug for Registers {
    /// Debug representation of registers
    /// 
    /// Provides detailed information for debugging, including
    /// both hex and decimal representations.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registers")
            .field("eax", &format_args!("0x{:08X} ({})", self.eax, self.eax))
            .field("ebx", &format_args!("0x{:08X} ({})", self.ebx, self.ebx))
            .field("ecx", &format_args!("0x{:08X} ({})", self.ecx, self.ecx))
            .field("edx", &format_args!("0x{:08X} ({})", self.edx, self.edx))
            .field("esi", &format_args!("0x{:08X} ({})", self.esi, self.esi))
            .field("edi", &format_args!("0x{:08X} ({})", self.edi, self.edi))
            .field("ebp", &format_args!("0x{:08X} ({})", self.ebp, self.ebp))
            .field("esp", &format_args!("0x{:08X} ({})", self.esp, self.esp))
            .field("eip", &format_args!("0x{:08X} ({})", self.eip, self.eip))
            .finish()
    }
}
```

**Debug implementation details**:

1. **`debug_struct()`**: Creates structured debug output
2. **`format_args!`**: Lazy formatting (only computed if needed)
3. **Hex + decimal**: Shows both representations for clarity
4. **`finish()`**: Completes the debug struct

### Step 9: Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_registers() {
        let regs = Registers::new();
        
        // Check default values
        assert_eq!(regs.eax, 0);
        assert_eq!(regs.ebx, 0);
        assert_eq!(regs.ecx, 0);
        assert_eq!(regs.edx, 0);
        assert_eq!(regs.esi, 0);
        assert_eq!(regs.edi, 0);
        
        // Check special register defaults
        assert_eq!(regs.ebp, 0xFFFF_0000);
        assert_eq!(regs.esp, 0xFFFF_0000);
        assert_eq!(regs.eip, 0x0000_1000);
    }
    
    #[test]
    fn test_register_access() {
        let mut regs = Registers::new();
        
        // Test getter
        assert_eq!(regs.get(RegisterName::EAX), 0);
        
        // Test setter
        regs.set(RegisterName::EAX, 0x12345678);
        assert_eq!(regs.get(RegisterName::EAX), 0x12345678);
        assert_eq!(regs.eax, 0x12345678);
        
        // Test all registers
        regs.set(RegisterName::EBX, 0x87654321);
        regs.set(RegisterName::ECX, 0x11111111);
        regs.set(RegisterName::EDX, 0x22222222);
        
        assert_eq!(regs.get(RegisterName::EBX), 0x87654321);
        assert_eq!(regs.get(RegisterName::ECX), 0x11111111);
        assert_eq!(regs.get(RegisterName::EDX), 0x22222222);
    }
    
    #[test]
    fn test_instruction_pointer_operations() {
        let mut regs = Registers::new();
        
        // Test initial value
        assert_eq!(regs.instruction_pointer(), 0x0000_1000);
        
        // Test setting
        regs.set_instruction_pointer(0x2000);
        assert_eq!(regs.instruction_pointer(), 0x2000);
        assert_eq!(regs.eip, 0x2000);
        
        // Test advancing
        regs.advance_ip(4);
        assert_eq!(regs.instruction_pointer(), 0x2004);
        
        // Test wrapping (if we ever hit 32-bit limit)
        regs.set_instruction_pointer(0xFFFFFFFE);
        regs.advance_ip(4);
        assert_eq!(regs.instruction_pointer(), 0x00000002); // Wrapped!
    }
    
    #[test]
    fn test_stack_pointer_operations() {
        let mut regs = Registers::new();
        
        // Test initial value
        assert_eq!(regs.stack_pointer(), 0xFFFF_0000);
        
        // Test setting
        regs.set_stack_pointer(0xFFFE_0000);
        assert_eq!(regs.stack_pointer(), 0xFFFE_0000);
        assert_eq!(regs.esp, 0xFFFE_0000);
    }
    
    #[test]
    fn test_register_type_checking() {
        let regs = Registers::new();
        
        // Test stack pointer detection
        assert!(regs.is_stack_pointer(RegisterName::ESP));
        assert!(!regs.is_stack_pointer(RegisterName::EAX));
        
        // Test instruction pointer detection
        assert!(regs.is_instruction_pointer(RegisterName::EIP));
        assert!(!regs.is_instruction_pointer(RegisterName::EAX));
    }
    
    #[test]
    fn test_reset() {
        let mut regs = Registers::new();
        
        // Modify some registers
        regs.eax = 0x12345678;
        regs.ebx = 0x87654321;
        regs.eip = 0x5000;
        
        // Reset
        regs.reset();
        
        // Check they're back to defaults
        assert_eq!(regs.eax, 0);
        assert_eq!(regs.ebx, 0);
        assert_eq!(regs.eip, 0x0000_1000);
    }
    
    #[test]
    fn test_display_formatting() {
        let regs = Registers::new();
        let formatted = format!("{}", regs);
        
        // Check that all registers are present
        assert!(formatted.contains("EAX: 0x00000000"));
        assert!(formatted.contains("ESP: 0xFFFF0000"));
        assert!(formatted.contains("EIP: 0x00001000"));
    }
}
```

**Test explanations**:

1. **`test_new_registers()`**: Verify default values
2. **`test_register_access()`**: Test getter/setter methods
3. **`test_instruction_pointer_operations()`**: Test EIP manipulation
4. **`test_stack_pointer_operations()`**: Test ESP manipulation
5. **`test_register_type_checking()`**: Test special register detection
6. **`test_reset()`**: Test reset functionality
7. **`test_display_formatting()`**: Test output formatting

## Summary

This implementation provides:

1. **Type-safe register access** using enums
2. **Clear documentation** for every method
3. **Safe defaults** for CPU initialization
4. **Helper methods** for common operations
5. **Comprehensive testing** for all functionality
6. **Debug support** for development

