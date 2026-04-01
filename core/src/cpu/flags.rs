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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    /// Carry Flag (CF) - Bit 0
    /// Set when unsigned arithmetic overflows (carry out of MSB)
    pub cf: bool,
    
    /// Parity Flag (PF) - Bit 2  
    /// Set when the lowest byte has an even number of 1-bits
    pub pf: bool,
    
    /// Auxiliary Carry Flag (AF) - Bit 4
    /// Set when there's a carry from bit 3 to bit 4
    pub af: bool,
    
    /// Zero Flag (ZF) - Bit 6
    /// Set when the result of an operation equals zero
    pub zf: bool,
    
    /// Sign Flag (SF) - Bit 7
    /// Set when the result is negative (bit 31 = 1)
    pub sf: bool,
    
    /// Overflow Flag (OF) - Bit 11
    /// Set when signed arithmetic overflows
    pub of: bool,
}

impl Flags {
    /// Create a new Flags struct with all flags cleared
    /// 
    /// Default state: All flags are false (cleared)
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
    
    /// Calculate flags for an addition operation
    /// 
    /// This method implements the flag calculation logic for ADD instructions.
    /// It calculates all six flags based on the operands and result.
    /// 
    /// # Arguments
    /// * `a` - First operand (destination)
    /// * `b` - Second operand (source)  
    /// * `result` - The result of the addition (a + b)
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
        self.af = ((a ^ b ^ result) & 0x10) != 0;
    }
    
    /// Calculate flags for a subtraction operation
    /// 
    /// This method implements the flag calculation logic for SUB instructions.
    /// 
    /// # Arguments
    /// * `a` - First operand (destination)
    /// * `b` - Second operand (source)
    /// * `result` - The result of the subtraction (a - b)
    pub fn calculate_sub_flags(&mut self, a: u32, b: u32, result: u32) {
        // CF (Carry Flag): Borrow occurred
        // If a < b, we had to borrow
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
        self.af = ((a ^ b ^ result) & 0x10) != 0;
    }
    
    /// Calculate flags for a logical AND operation
    /// 
    /// AND operations only affect ZF, SF, and PF (not CF, OF, AF).
    /// 
    /// # Arguments
    /// * `result` - The result of the AND operation
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
    /// OR operations only affect ZF, SF, and PF (not CF, OF, AF).
    ///
    /// # Arguments
    /// * `result` - The result of the OR operation
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
    
    /// Clear all flags
    pub fn clear_all(&mut self) {
        self.cf = false;
        self.pf = false;
        self.af = false;
        self.zf = false;
        self.sf = false;
        self.of = false;
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Flags {
    /// Format flags for human-readable output
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
    fn test_add_flags_simple() {
        let mut flags = Flags::new();
        
        // Simple addition: 5 + 3 = 8
        flags.calculate_add_flags(5, 3, 8);
        
        assert!(!flags.cf);  // No carry
        assert!(!flags.zf);  // Not zero
        assert!(!flags.sf);  // Not negative
        assert!(!flags.of);  // No overflow
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
    }
    
    #[test]
    fn test_logical_flags() {
        let mut flags = Flags::new();
        
        // AND operation: 0x12345678 & 0x0000FFFF = 0x00005678
        flags.calculate_and_flags(0x00005678);
        
        assert!(!flags.cf);  // Always cleared for AND
        assert!(!flags.of);  // Always cleared for AND
        assert!(!flags.zf);  // Not zero
        assert!(!flags.sf);  // Not negative
    }

    #[test]
    fn test_or_flags() {
        let mut flags = Flags::new();

        // OR operation: 0x12340000 | 0x00005678 = 0x12345678
        flags.calculate_or_flags(0x12345678);

        assert!(!flags.cf);  // Always cleared for OR
        assert!(!flags.of);  // Always cleared for OR
        assert!(!flags.zf);  // Not zero
        assert!(!flags.sf);  // Not negative
    }
    
    #[test]
    fn test_clear_all() {
        let mut flags = Flags::new();
        
        // Set some flags
        flags.cf = true;
        flags.zf = true;
        flags.sf = true;
        
        // Clear all
        flags.clear_all();
        
        assert!(!flags.cf);
        assert!(!flags.zf);
        assert!(!flags.sf);
    }
}
