# Implementing the Flags Module

## Overview
This document provides a step-by-step guide to implementing the CPU flags in Rust. Every line of code is explained with the underlying concepts, flag calculation algorithms, and reasoning.

## File: `core/src/cpu/flags.rs`

### Step 1: Module Declaration and Imports

```rust
//! CPU Flags Module
//! 
//! This module implements the x86-32 EFLAGS register.
//! Flags are single bits that indicate the result or status of CPU operations.
//! 
//! Key concepts:
//! - Flags are status indicators (tell you ABOUT the result, not the result itself)
//! - Most arithmetic/logic instructions automatically update flags
//! - Flags enable conditional jumps and decision making
//! - Each flag is a single bit (true/false, set/clear, 1/0)
//! - We use individual booleans for clarity (not bitfields)

use std::fmt;
```

**Explanation**:
- `//!` creates module-level documentation
- Explains the purpose and key concepts
- `use std::fmt;` for implementing `Display` trait

### Step 2: Flags Struct Definition

```rust
/// CPU Flags Structure
/// 
/// Represents the EFLAGS register as individual boolean fields.
/// Each flag is a single bit that indicates the result of operations.
/// 
/// Design choice: Individual booleans vs bitfield
/// - Booleans: Easier to read, modify, and debug
/// - Bitfield: More memory efficient, matches hardware exactly
/// 
/// We choose booleans for clarity and maintainability.
/// 
/// Memory layout:
/// - Each bool takes 1 byte (not 1 bit due to alignment)
/// - Total size: 6 flags × 1 byte = 6 bytes
/// - Could be optimized to 1 byte with bitfields later
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    /// Carry Flag (CF) - Bit 0
    /// Set when unsigned arithmetic overflows (carry out of MSB)
    /// Used for: Multi-precision arithmetic, unsigned comparisons
    pub cf: bool,
    
    /// Parity Flag (PF) - Bit 2  
    /// Set when the lowest byte has an even number of 1-bits
    /// Used for: Error detection in communication protocols (rarely used)
    pub pf: bool,
    
    /// Auxiliary Carry Flag (AF) - Bit 4
    /// Set when there's a carry from bit 3 to bit 4
    /// Used for: BCD arithmetic with DAA/DAS instructions (rarely used)
    pub af: bool,
    
    /// Zero Flag (ZF) - Bit 6
    /// Set when the result of an operation equals zero
    /// Used for: Equality testing, loop termination (most commonly used!)
    pub zf: bool,
    
    /// Sign Flag (SF) - Bit 7
    /// Set when the result is negative (bit 31 = 1)
    /// Used for: Signed comparisons, negative number detection
    pub sf: bool,
    
    /// Overflow Flag (OF) - Bit 11
    /// Set when signed arithmetic overflows
    /// Used for: Signed comparisons, error detection in signed arithmetic
    pub of: bool,
}
```

**Design decisions explained**:

1. **`pub` fields**: Public for simplicity, can add getters/setters later
2. **`bool` type**: Clear true/false, easy to work with
3. **Field order**: Grouped by bit position for reference
4. **Detailed comments**: Each flag explains its purpose and usage
5. **Derive traits**: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` for convenience

**Why not bitfield?**
- **Clarity**: `flags.cf = true` is clearer than bit manipulation
- **Debugging**: Easy to inspect individual flags
- **Maintainability**: Less error-prone than bitwise operations
- **Performance**: Modern CPUs handle bools efficiently

### Step 3: Constructor Implementation

```rust
impl Flags {
    /// Create a new Flags struct with all flags cleared
    /// 
    /// Default state: All flags are false (cleared)
    /// This represents a "clean" CPU state with no previous operation results.
    /// 
    /// # Returns
    /// A new Flags instance with all flags cleared
    /// 
    /// # Example
    /// ```rust
    /// let flags = Flags::new();
    /// assert!(!flags.cf && !flags.zf && !flags.sf);
    /// ```
    pub fn new() -> Self {
        Flags {
            cf: false,  // No carry
            pf: false,  // No parity
            af: false,  // No auxiliary carry
            zf: false,  // Not zero
            sf: false,  // Not negative
            of: false,  // No overflow
        }
    }
    
    /// Create Flags from a 32-bit EFLAGS value
    /// 
    /// This method extracts individual flags from a packed EFLAGS register.
    /// Useful for restoring CPU state or implementing PUSHF/POPF instructions.
    /// 
    /// # Arguments
    /// * `eflags` - The 32-bit EFLAGS value
    /// 
    /// # Returns
    /// A Flags struct with flags extracted from the EFLAGS value
    /// 
    /// # Example
    /// ```rust
    /// let eflags = 0x00000246;  // CF=0, PF=1, AF=1, ZF=1, SF=0, OF=0
    /// let flags = Flags::from_eflags(eflags);
    /// assert!(flags.pf && flags.af && flags.zf);
    /// ```
    pub fn from_eflags(eflags: u32) -> Self {
        Flags {
            cf: (eflags & 0x00000001) != 0,  // Bit 0
            pf: (eflags & 0x00000004) != 0,  // Bit 2
            af: (eflags & 0x00000010) != 0,  // Bit 4
            zf: (eflags & 0x00000040) != 0,  // Bit 6
            sf: (eflags & 0x00000080) != 0,  // Bit 7
            of: (eflags & 0x00000800) != 0,  // Bit 11
        }
    }
    
    /// Convert Flags to a 32-bit EFLAGS value
    /// 
    /// This method packs individual flags into a 32-bit EFLAGS register.
    /// Useful for saving CPU state or implementing PUSHF/POPF instructions.
    /// 
    /// # Returns
    /// A 32-bit value representing the EFLAGS register
    /// 
    /// # Example
    /// ```rust
    /// let mut flags = Flags::new();
    /// flags.cf = true;
    /// flags.zf = true;
    /// let eflags = flags.to_eflags();
    /// assert_eq!(eflags & 0x00000041, 0x00000041);  // CF and ZF set
    /// ```
    pub fn to_eflags(&self) -> u32 {
        let mut eflags = 0u32;
        
        if self.cf { eflags |= 0x00000001; }  // Bit 0
        if self.pf { eflags |= 0x00000004; }  // Bit 2
        if self.af { eflags |= 0x00000010; }  // Bit 4
        if self.zf { eflags |= 0x00000040; }  // Bit 6
        if self.sf { eflags |= 0x00000080; }  // Bit 7
        if self.of { eflags |= 0x00000800; }  // Bit 11
        
        eflags
    }
}
```

**Constructor explanations**:

1. **`new()`**: Clean state with all flags cleared
2. **`from_eflags()`**: Extract flags from packed EFLAGS value
3. **`to_eflags()`**: Pack flags into EFLAGS value

**Bit manipulation details**:
- `eflags & 0x00000001` extracts bit 0 (CF)
- `!= 0` converts to boolean (true if bit is set)
- `|=` sets bits in the result

### Step 4: Flag Calculation Methods

```rust
impl Flags {
    /// Calculate flags for an addition operation
    /// 
    /// This method implements the flag calculation logic for ADD instructions.
    /// It calculates all six flags based on the operands and result.
    /// 
    /// # Arguments
    /// * `a` - First operand (destination)
    /// * `b` - Second operand (source)  
    /// * `result` - The result of the addition (a + b)
    /// 
    /// # Example
    /// ```rust
    /// let mut flags = Flags::new();
    /// flags.calculate_add_flags(0x7FFFFFFF, 1, 0x80000000);
    /// assert!(flags.of);  // Signed overflow occurred
    /// assert!(flags.sf);  // Result is negative
    /// ```
    pub fn calculate_add_flags(&mut self, a: u32, b: u32, result: u32) {
        // CF (Carry Flag): Unsigned overflow
        // If result < a, we wrapped around (carried out of bit 31)
        self.cf = result < a;
        
        // ZF (Zero Flag): Result is zero
        self.zf = result == 0;
        
        // SF (Sign Flag): Result is negative (bit 31 is set)
        self.sf = (result & 0x8000_0000) != 0;
        
        // OF (Overflow Flag): Signed overflow
        // Overflow occurs when:
        // - Both operands have same sign AND
        // - Result has different sign
        let operand_signs_match = (a ^ b) & 0x8000_0000 == 0;
        let result_sign_differs = (a ^ result) & 0x8000_0000 != 0;
        self.of = operand_signs_match && result_sign_differs;
        
        // PF (Parity Flag): Even number of 1-bits in lowest byte
        let lowest_byte = (result & 0xFF) as u8;
        self.pf = (lowest_byte.count_ones() % 2) == 0;
        
        // AF (Auxiliary Carry Flag): Carry from bit 3 to bit 4
        // This is complex to calculate directly, so we use a common algorithm:
        // AF = ((a ^ b ^ result) & 0x10) != 0
        self.af = ((a ^ b ^ result) & 0x10) != 0;
    }
    
    /// Calculate flags for a subtraction operation
    /// 
    /// This method implements the flag calculation logic for SUB instructions.
    /// Subtraction is implemented as addition with two's complement.
    /// 
    /// # Arguments
    /// * `a` - First operand (destination)
    /// * `b` - Second operand (source)
    /// * `result` - The result of the subtraction (a - b)
    /// 
    /// # Example
    /// ```rust
    /// let mut flags = Flags::new();
    /// flags.calculate_sub_flags(5, 10, 0xFFFFFFFB);  // 5 - 10 = -5
    /// assert!(flags.cf);  // Borrow occurred (5 < 10)
    /// assert!(flags.sf);  // Result is negative
    /// ```
    pub fn calculate_sub_flags(&mut self, a: u32, b: u32, result: u32) {
        // CF (Carry Flag): Borrow occurred
        // If a < b, we had to borrow (result would be negative in unsigned)
        self.cf = a < b;
        
        // ZF (Zero Flag): Result is zero
        self.zf = result == 0;
        
        // SF (Sign Flag): Result is negative (bit 31 is set)
        self.sf = (result & 0x8000_0000) != 0;
        
        // OF (Overflow Flag): Signed overflow
        // For subtraction: overflow when signs of operands differ
        // and result sign differs from first operand
        let operand_signs_differ = (a ^ b) & 0x8000_0000 != 0;
        let result_sign_differs = (a ^ result) & 0x8000_0000 != 0;
        self.of = operand_signs_differ && result_sign_differs;
        
        // PF (Parity Flag): Even number of 1-bits in lowest byte
        let lowest_byte = (result & 0xFF) as u8;
        self.pf = (lowest_byte.count_ones() % 2) == 0;
        
        // AF (Auxiliary Carry Flag): Borrow from bit 4
        // For subtraction: AF = ((a ^ b ^ result) & 0x10) != 0
        self.af = ((a ^ b ^ result) & 0x10) != 0;
    }
    
    /// Calculate flags for a logical AND operation
    /// 
    /// This method implements the flag calculation logic for AND instructions.
    /// AND operations only affect ZF, SF, and PF (not CF, OF, AF).
    /// 
    /// # Arguments
    /// * `result` - The result of the AND operation
    /// 
    /// # Example
    /// ```rust
    /// let mut flags = Flags::new();
    /// flags.calculate_and_flags(0x12345678 & 0x0000FFFF);
    /// assert!(flags.zf);  // Result is zero
    /// ```
    pub fn calculate_and_flags(&mut self, result: u32) {
        // CF and OF are always cleared for AND
        self.cf = false;
        self.of = false;
        
        // AF is undefined for AND (we'll clear it)
        self.af = false;
        
        // ZF (Zero Flag): Result is zero
        self.zf = result == 0;
        
        // SF (Sign Flag): Result is negative (bit 31 is set)
        self.sf = (result & 0x8000_0000) != 0;
        
        // PF (Parity Flag): Even number of 1-bits in lowest byte
        let lowest_byte = (result & 0xFF) as u8;
        self.pf = (lowest_byte.count_ones() % 2) == 0;
    }
    
    /// Calculate flags for a logical OR operation
    /// 
    /// This method implements the flag calculation logic for OR instructions.
    /// OR operations only affect ZF, SF, and PF (not CF, OF, AF).
    /// 
    /// # Arguments
    /// * `result` - The result of the OR operation
    /// 
    /// # Example
    /// ```rust
    /// let mut flags = Flags::new();
    /// flags.calculate_or_flags(0x00000000 | 0x00000001);
    /// assert!(!flags.zf);  // Result is not zero
    /// ```
    pub fn calculate_or_flags(&mut self, result: u32) {
        // CF and OF are always cleared for OR
        self.cf = false;
        self.of = false;
        
        // AF is undefined for OR (we'll clear it)
        self.af = false;
        
        // ZF (Zero Flag): Result is zero
        self.zf = result == 0;
        
        // SF (Sign Flag): Result is negative (bit 31 is set)
        self.sf = (result & 0x8000_0000) != 0;
        
        // PF (Parity Flag): Even number of 1-bits in lowest byte
        let lowest_byte = (result & 0xFF) as u8;
        self.pf = (lowest_byte.count_ones() % 2) == 0;
    }
    
    /// Calculate flags for a logical XOR operation
    /// 
    /// This method implements the flag calculation logic for XOR instructions.
    /// XOR operations only affect ZF, SF, and PF (not CF, OF, AF).
    /// 
    /// # Arguments
    /// * `result` - The result of the XOR operation
    /// 
    /// # Example
    /// ```rust
    /// let mut flags = Flags::new();
    /// flags.calculate_xor_flags(0x12345678 ^ 0x12345678);
    /// assert!(flags.zf);  // Result is zero (same values XORed)
    /// ```
    pub fn calculate_xor_flags(&mut self, result: u32) {
        // CF and OF are always cleared for XOR
        self.cf = false;
        self.of = false;
        
        // AF is undefined for XOR (we'll clear it)
        self.af = false;
        
        // ZF (Zero Flag): Result is zero
        self.zf = result == 0;
        
        // SF (Sign Flag): Result is negative (bit 31 is set)
        self.sf = (result & 0x8000_0000) != 0;
        
        // PF (Parity Flag): Even number of 1-bits in lowest byte
        let lowest_byte = (result & 0xFF) as u8;
        self.pf = (lowest_byte.count_ones() % 2) == 0;
    }
}
```

**Flag calculation explanations**:

1. **CF (Carry Flag)**:
   - **Addition**: `result < a` means we wrapped around
   - **Subtraction**: `a < b` means we had to borrow

2. **ZF (Zero Flag)**: Simple - `result == 0`

3. **SF (Sign Flag)**: Check bit 31 - `(result & 0x8000_0000) != 0`

4. **OF (Overflow Flag)**:
   - **Addition**: Same sign operands → different sign result
   - **Subtraction**: Different sign operands → different sign result

5. **PF (Parity Flag)**: Count 1-bits in lowest byte, check if even

6. **AF (Auxiliary Carry)**: Complex algorithm using XOR operations

### Step 5: Helper Methods

```rust
impl Flags {
    /// Clear all flags
    /// 
    /// Sets all flags to false (cleared state).
    /// Useful for resetting CPU state or before operations that don't affect flags.
    /// 
    /// # Example
    /// ```rust
    /// let mut flags = Flags::new();
    /// flags.cf = true;
    /// flags.zf = true;
    /// flags.clear_all();
    /// assert!(!flags.cf && !flags.zf);
    /// ```
    pub fn clear_all(&mut self) {
        self.cf = false;
        self.pf = false;
        self.af = false;
        self.zf = false;
        self.sf = false;
        self.of = false;
    }
    
    /// Clear specific flags
    /// 
    /// Sets the specified flags to false.
    /// Useful for operations that only affect certain flags.
    /// 
    /// # Arguments
    /// * `flags_to_clear` - Bitmask of flags to clear
    /// 
    /// # Example
    /// ```rust
    /// let mut flags = Flags::new();
    /// flags.cf = true;
    /// flags.zf = true;
    /// flags.clear_flags(0x00000041);  // Clear CF and ZF
    /// assert!(!flags.cf && !flags.zf);
    /// ```
    pub fn clear_flags(&mut self, flags_to_clear: u32) {
        if (flags_to_clear & 0x00000001) != 0 { self.cf = false; }
        if (flags_to_clear & 0x00000004) != 0 { self.pf = false; }
        if (flags_to_clear & 0x00000010) != 0 { self.af = false; }
        if (flags_to_clear & 0x00000040) != 0 { self.zf = false; }
        if (flags_to_clear & 0x00000080) != 0 { self.sf = false; }
        if (flags_to_clear & 0x00000800) != 0 { self.of = false; }
    }
    
    /// Check if any flags are set
    /// 
    /// Returns true if any flag is currently set (true).
    /// Useful for debugging or status checking.
    /// 
    /// # Returns
    /// True if any flag is set, false if all flags are cleared
    /// 
    /// # Example
    /// ```rust
    /// let mut flags = Flags::new();
    /// assert!(!flags.any_set());
    /// flags.zf = true;
    /// assert!(flags.any_set());
    /// ```
    pub fn any_set(&self) -> bool {
        self.cf || self.pf || self.af || self.zf || self.sf || self.of
    }
    
    /// Get a summary string of all flags
    /// 
    /// Returns a string showing which flags are set.
    /// Useful for debugging and disassembly output.
    /// 
    /// # Returns
    /// A string like "CF ZF" or "none" if no flags are set
    /// 
    /// # Example
    /// ```rust
    /// let mut flags = Flags::new();
    /// flags.cf = true;
    /// flags.zf = true;
    /// assert_eq!(flags.summary(), "CF ZF");
    /// ```
    pub fn summary(&self) -> String {
        let mut flags = Vec::new();
        
        if self.cf { flags.push("CF"); }
        if self.pf { flags.push("PF"); }
        if self.af { flags.push("AF"); }
        if self.zf { flags.push("ZF"); }
        if self.sf { flags.push("SF"); }
        if self.of { flags.push("OF"); }
        
        if flags.is_empty() {
            "none".to_string()
        } else {
            flags.join(" ")
        }
    }
}
```

**Helper method explanations**:

1. **`clear_all()`**: Reset all flags to false
2. **`clear_flags()`**: Clear specific flags using bitmask
3. **`any_set()`**: Check if any flags are currently set
4. **`summary()`**: Human-readable flag status

### Step 6: Display Implementation

```rust
impl fmt::Display for Flags {
    /// Format flags for human-readable output
    /// 
    /// Shows all flags in a compact format with their current state.
    /// Set flags are shown in uppercase, cleared flags in lowercase.
    /// 
    /// # Example
    /// ```
    /// let mut flags = Flags::new();
    /// flags.cf = true;
    /// flags.zf = true;
    /// println!("{}", flags);
    /// // Output: CF:1 PF:0 AF:0 ZF:1 SF:0 OF:0
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CF:{} PF:{} AF:{} ZF:{} SF:{} OF:{}",
               if self.cf { 1 } else { 0 },
               if self.pf { 1 } else { 0 },
               if self.af { 1 } else { 0 },
               if self.zf { 1 } else { 0 },
               if self.sf { 1 } else { 0 },
               if self.of { 1 } else { 0 })
    }
}
```

**Display implementation details**:

1. **Compact format**: Shows all flags in one line
2. **Numeric values**: 1 for set, 0 for cleared (easier to read than true/false)
3. **Consistent order**: CF, PF, AF, ZF, SF, OF (by bit position)

### Step 7: Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_flags() {
        let flags = Flags::new();
        
        // All flags should be cleared
        assert!(!flags.cf);
        assert!(!flags.pf);
        assert!(!flags.af);
        assert!(!flags.zf);
        assert!(!flags.sf);
        assert!(!flags.of);
    }
    
    #[test]
    fn test_eflags_conversion() {
        // Test round-trip conversion
        let mut flags = Flags::new();
        flags.cf = true;
        flags.zf = true;
        flags.of = true;
        
        let eflags = flags.to_eflags();
        let restored = Flags::from_eflags(eflags);
        
        assert_eq!(flags.cf, restored.cf);
        assert_eq!(flags.zf, restored.zf);
        assert_eq!(flags.of, restored.of);
        assert_eq!(flags.pf, restored.pf);
        assert_eq!(flags.af, restored.af);
        assert_eq!(flags.sf, restored.sf);
    }
    
    #[test]
    fn test_add_flags_simple() {
        let mut flags = Flags::new();
        
        // Simple addition: 5 + 3 = 8
        flags.calculate_add_flags(5, 3, 8);
        
        assert!(!flags.cf);  // No carry
        assert!(!flags.zf);  // Not zero
        assert!(!flags.sf);  // Not negative
        assert!(!flags.of);  // No overflow
        assert!(!flags.pf);  // Odd parity (8 = 1000 in binary)
        assert!(!flags.af);  // No auxiliary carry
    }
    
    #[test]
    fn test_add_flags_carry() {
        let mut flags = Flags::new();
        
        // Unsigned overflow: 0xFFFFFFFF + 1 = 0
        flags.calculate_add_flags(0xFFFFFFFF, 1, 0);
        
        assert!(flags.cf);   // Carry occurred
        assert!(flags.zf);   // Result is zero
        assert!(!flags.sf);  // Not negative
        assert!(!flags.of);  // No signed overflow
        assert!(flags.pf);   // Even parity (0 = 00000000)
        assert!(flags.af);   // Auxiliary carry occurred
    }
    
    #[test]
    fn test_add_flags_overflow() {
        let mut flags = Flags::new();
        
        // Signed overflow: 0x7FFFFFFF + 1 = 0x80000000
        flags.calculate_add_flags(0x7FFFFFFF, 1, 0x80000000);
        
        assert!(!flags.cf);  // No unsigned carry
        assert!(!flags.zf);  // Not zero
        assert!(flags.sf);   // Negative (bit 31 set)
        assert!(flags.of);   // Signed overflow occurred!
        assert!(flags.pf);   // Even parity (0x00 in lowest byte)
        assert!(!flags.af);  // No auxiliary carry
    }
    
    #[test]
    fn test_sub_flags_borrow() {
        let mut flags = Flags::new();
        
        // Subtraction with borrow: 5 - 10 = -5 (0xFFFFFFFB)
        flags.calculate_sub_flags(5, 10, 0xFFFFFFFB);
        
        assert!(flags.cf);   // Borrow occurred (5 < 10)
        assert!(!flags.zf);  // Not zero
        assert!(flags.sf);   // Negative
        assert!(!flags.of);  // No overflow
        assert!(!flags.pf);  // Odd parity (0xFB = 11111011)
        assert!(flags.af);   // Auxiliary borrow occurred
    }
    
    #[test]
    fn test_logical_flags() {
        let mut flags = Flags::new();
        
        // AND operation: 0x12345678 & 0x0000FFFF = 0x00005678
        flags.calculate_and_flags(0x00005678);
        
        assert!(!flags.cf);  // Always cleared for AND
        assert!(!flags.of);  // Always cleared for AND
        assert!(!flags.af);  // Always cleared for AND
        assert!(!flags.zf);  // Not zero
        assert!(!flags.sf);  // Not negative
        assert!(!flags.pf);  // Odd parity (0x78 = 01111000)
    }
    
    #[test]
    fn test_helper_methods() {
        let mut flags = Flags::new();
        
        // Test clear_all
        flags.cf = true;
        flags.zf = true;
        flags.clear_all();
        assert!(!flags.any_set());
        
        // Test any_set
        flags.cf = true;
        assert!(flags.any_set());
        
        // Test summary
        flags.zf = true;
        assert_eq!(flags.summary(), "CF ZF");
        
        // Test clear_flags
        flags.clear_flags(0x00000041);  // Clear CF and ZF
        assert!(!flags.cf && !flags.zf);
    }
    
    #[test]
    fn test_display_formatting() {
        let mut flags = Flags::new();
        flags.cf = true;
        flags.zf = true;
        
        let formatted = format!("{}", flags);
        assert!(formatted.contains("CF:1"));
        assert!(formatted.contains("ZF:1"));
        assert!(formatted.contains("PF:0"));
    }
}
```

**Test explanations**:

1. **`test_new_flags()`**: Verify default state
2. **`test_eflags_conversion()`**: Test round-trip conversion
3. **`test_add_flags_simple()`**: Test normal addition
4. **`test_add_flags_carry()`**: Test unsigned overflow
5. **`test_add_flags_overflow()`**: Test signed overflow
6. **`test_sub_flags_borrow()`**: Test subtraction with borrow
7. **`test_logical_flags()`**: Test AND operation flags
8. **`test_helper_methods()`**: Test utility methods
9. **`test_display_formatting()`**: Test output formatting

## Summary

This implementation provides:

1. **Complete flag calculation** for arithmetic and logical operations
2. **Clear documentation** for every method and algorithm
3. **EFLAGS conversion** for compatibility with real hardware
4. **Helper methods** for common flag operations
5. **Comprehensive testing** for all flag calculations
6. **Debug support** for development

