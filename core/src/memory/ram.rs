//! RAM and MMIO Implementation
//! 
//! This module implements the complete memory system including:
//! - RAM simulation using Vec<u8>
//! - Memory-mapped I/O (MMIO) for devices  
//! - Little-endian byte ordering
//! - Bounds checking and security

use std::fmt;

/// Memory access errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryError {
    /// Attempted to access memory outside valid range
    OutOfBounds { address: u32, max_address: u32 },
    
    /// Attempted to access NULL region (0x0000-0x0FFF)
    NullPointerAccess { address: u32 },
    
    /// Attempted to access invalid MMIO address
    InvalidMmioAddress { address: u32 },
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::OutOfBounds { address, max_address } => {
                write!(f, "Memory access out of bounds: address 0x{:08X}, max 0x{:08X}", 
                       address, max_address)
            },
            MemoryError::NullPointerAccess { address } => {
                write!(f, "NULL pointer access attempted: address 0x{:08X}", address)
            },
            MemoryError::InvalidMmioAddress { address } => {
                write!(f, "Invalid MMIO address: 0x{:08X}", address)
            },
        }
    }
}

impl std::error::Error for MemoryError {}

/// Memory-Mapped I/O Devices
/// 
/// Device map (from project specification):
/// - 0xFFFF0000: Switch register (8-bit read)
/// - 0xFFFF0004: LED register (8-bit write)  
/// - 0xFFFF0008: 7-segment display (8-bit write)
/// - 0xFFFF000C: I/O control register (8-bit read/write)
#[derive(Debug, Clone)]
pub struct IoDevices {
    /// Switch states (8 switches, each bit represents one switch)
    switches: u8,
    
    /// LED states (8 LEDs, each bit represents one LED)
    leds: u8,
    
    /// 7-segment display value
    seven_segment: u8,
    
    /// I/O control register
    io_ctrl: u8,
}

impl IoDevices {
    /// Create new I/O devices with default state
    pub fn new() -> Self {
        IoDevices {
            switches: 0x00,        // All switches off
            leds: 0x00,            // All LEDs off
            seven_segment: 0x3F,   // Display '0'
            io_ctrl: 0x00,         // Default control state
        }
    }
    
    /// Read from MMIO address
    pub fn read(&self, address: u32) -> Result<u8, MemoryError> {
        match address {
            0xFFFF0000 => Ok(self.switches),
            0xFFFF000C => Ok(self.io_ctrl),
            _ => Err(MemoryError::InvalidMmioAddress { address }),
        }
    }
    
    /// Write to MMIO address
    pub fn write(&mut self, address: u32, value: u8) -> Result<(), MemoryError> {
        match address {
            0xFFFF0004 => {
                self.leds = value;
                Ok(())
            },
            0xFFFF0008 => {
                self.seven_segment = value;
                Ok(())
            },
            0xFFFF000C => {
                self.io_ctrl = value;
                Ok(())
            },
            _ => Err(MemoryError::InvalidMmioAddress { address }),
        }
    }
    
    /// Get current LED state
    pub fn get_leds(&self) -> u8 {
        self.leds
    }
    
    /// Get current 7-segment display value
    pub fn get_seven_segment(&self) -> u8 {
        self.seven_segment
    }
    
    /// Set switch states (for testing/simulation)
    pub fn set_switches(&mut self, switches: u8) {
        self.switches = switches;
    }
}

/// CPU Memory Structure
/// 
/// Memory layout:
/// - 0x00000000-0x00000FFF: Reserved (NULL region)
/// - 0x00001000-0xFFFEFFFF: RAM
/// - 0xFFFF0000-0xFFFFFFFF: MMIO devices
#[derive(Debug, Clone)]
pub struct Memory {
    /// Main RAM - vector of bytes
    ram: Vec<u8>,
    
    /// Memory-mapped I/O devices
    mmio: IoDevices,
    
    /// Memory size in bytes
    size: usize,
}

impl Memory {
    /// Create new memory with specified size
    /// 
    /// # Arguments
    /// * `size` - Total memory size in bytes
    pub fn new(size: usize) -> Self {
        Memory {
            ram: vec![0; size],
            mmio: IoDevices::new(),
            size,
        }
    }
    
    /// Create memory with default size (16 MB)
    pub fn default() -> Self {
        Self::new(16 * 1024 * 1024)
    }
    
    /// Check if address is in MMIO region
    fn is_mmio_address(&self, address: u32) -> bool {
        address >= 0xFFFF_0000
    }
    
    /// Check if address is in NULL region
    fn is_null_address(&self, address: u32) -> bool {
        address < 0x1000
    }
    
    /// Check if address is in valid RAM range
    fn is_valid_ram_address(&self, address: u32) -> bool {
        (address as usize) < self.size
    }
    
    /// Read a single byte from memory
    pub fn read_u8(&self, address: u32) -> Result<u8, MemoryError> {
        // Check for NULL pointer access
        if self.is_null_address(address) {
            return Err(MemoryError::NullPointerAccess { address });
        }
        
        // Handle MMIO
        if self.is_mmio_address(address) {
            return self.mmio.read(address);
        }
        
        // Check bounds for RAM
        if !self.is_valid_ram_address(address) {
            return Err(MemoryError::OutOfBounds {
                address,
                max_address: self.size as u32 - 1,
            });
        }
        
        // Read from RAM
        Ok(self.ram[address as usize])
    }
    
    /// Read a 32-bit double word from memory (little-endian)
    pub fn read_u32(&self, address: u32) -> Result<u32, MemoryError> {
        // Check for NULL pointer access
        if self.is_null_address(address) {
            return Err(MemoryError::NullPointerAccess { address });
        }
        
        // Handle MMIO (not supported for multi-byte reads)
        if self.is_mmio_address(address) {
            return Err(MemoryError::InvalidMmioAddress { address });
        }
        
        // Check bounds (need 4 bytes)
        if !self.is_valid_ram_address(address + 3) {
            return Err(MemoryError::OutOfBounds {
                address,
                max_address: self.size as u32 - 1,
            });
        }
        
        // Read bytes in little-endian order
        let byte0 = self.ram[address as usize] as u32;
        let byte1 = self.ram[(address + 1) as usize] as u32;
        let byte2 = self.ram[(address + 2) as usize] as u32;
        let byte3 = self.ram[(address + 3) as usize] as u32;
        
        // Combine bytes: byte3 is MSB, byte0 is LSB
        Ok((byte3 << 24) | (byte2 << 16) | (byte1 << 8) | byte0)
    }
    
    /// Write a single byte to memory
    pub fn write_u8(&mut self, address: u32, value: u8) -> Result<(), MemoryError> {
        // Check for NULL pointer access
        if self.is_null_address(address) {
            return Err(MemoryError::NullPointerAccess { address });
        }
        
        // Handle MMIO
        if self.is_mmio_address(address) {
            return self.mmio.write(address, value);
        }
        
        // Check bounds for RAM
        if !self.is_valid_ram_address(address) {
            return Err(MemoryError::OutOfBounds {
                address,
                max_address: self.size as u32 - 1,
            });
        }
        
        // Write to RAM
        self.ram[address as usize] = value;
        Ok(())
    }
    
    /// Write a 32-bit double word to memory (little-endian)
    pub fn write_u32(&mut self, address: u32, value: u32) -> Result<(), MemoryError> {
        // Check for NULL pointer access
        if self.is_null_address(address) {
            return Err(MemoryError::NullPointerAccess { address });
        }
        
        // Handle MMIO (not supported for multi-byte writes)
        if self.is_mmio_address(address) {
            return Err(MemoryError::InvalidMmioAddress { address });
        }
        
        // Check bounds (need 4 bytes)
        if !self.is_valid_ram_address(address + 3) {
            return Err(MemoryError::OutOfBounds {
                address,
                max_address: self.size as u32 - 1,
            });
        }
        
        // Write bytes in little-endian order
        self.ram[address as usize] = (value & 0xFF) as u8;           // Byte 0 (LSB)
        self.ram[(address + 1) as usize] = ((value >> 8) & 0xFF) as u8;   // Byte 1
        self.ram[(address + 2) as usize] = ((value >> 16) & 0xFF) as u8;  // Byte 2
        self.ram[(address + 3) as usize] = ((value >> 24) & 0xFF) as u8;  // Byte 3 (MSB)
        
        Ok(())
    }
    
    /// Push a 32-bit value onto the stack
    /// 
    /// # Arguments
    /// * `esp` - Current stack pointer (will be updated)
    /// * `value` - The value to push
    /// 
    /// # Returns
    /// New stack pointer value
    pub fn push_u32(&mut self, esp: u32, value: u32) -> Result<u32, MemoryError> {
        // Decrement ESP (stack grows downward)
        let new_esp = esp.wrapping_sub(4);
        
        // Write value to stack
        self.write_u32(new_esp, value)?;
        
        Ok(new_esp)
    }
    
    /// Pop a 32-bit value from the stack
    /// 
    /// # Arguments
    /// * `esp` - Current stack pointer (will be updated)
    /// 
    /// # Returns
    /// Tuple of (value, new_stack_pointer)
    pub fn pop_u32(&mut self, esp: u32) -> Result<(u32, u32), MemoryError> {
        // Read value from stack
        let value = self.read_u32(esp)?;
        
        // Increment ESP (stack shrinks upward)
        let new_esp = esp.wrapping_add(4);
        
        Ok((value, new_esp))
    }
    
    /// Get MMIO device reference
    pub fn get_mmio(&mut self) -> &mut IoDevices {
        &mut self.mmio
    }

    /// Get memory size in bytes
    pub fn size(&self) -> usize {
        self.size
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_pointer_access_error_message() {
        let mem = Memory::default();
        let addr = 0x000; // NULL region
        let err = mem.read_u8(addr).unwrap_err();
        assert!(format!("{}", err).contains("NULL pointer access"));
    }

    #[test]
    fn test_invalid_mmio_address_error_message() {
        let mem = Memory::default();
        let addr = 0xFFFF0010; // Not a valid MMIO address
        let err = mem.read_u8(addr).unwrap_err();
        assert!(format!("{}", err).contains("Invalid MMIO address"));
    }

    #[test]
    fn test_out_of_bounds_write_error() {
        let mut mem = Memory::default();
        let addr = mem.size() as u32; // definitely out of bounds
        let err = mem.write_u8(addr, 0xAA).unwrap_err();
        assert!(format!("{}", err).contains("out of bounds"));
    }

    #[test]
    fn test_null_pointer_write_error() {
        let mut mem = Memory::default();
        let addr = 0x000; // NULL region
        let err = mem.write_u8(addr, 0xAA).unwrap_err();
        assert!(format!("{}", err).contains("NULL pointer access"));
    }

    #[test]
    fn test_invalid_mmio_write_error() {
        let mut mem = Memory::default();
        let addr = 0xFFFF0010; // Not a valid MMIO address
        let err = mem.write_u8(addr, 0xAA).unwrap_err();
        assert!(format!("{}", err).contains("Invalid MMIO address"));
    }

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(1024);  // 1 KB for testing
        assert_eq!(memory.size, 1024);
    }

    #[test]
    fn test_read_write_u8() {
        let mut memory = Memory::new(0x10000);  // 64KB

        // Test normal write/read
        memory.write_u8(0x1000, 0x42).unwrap();
        assert_eq!(memory.read_u8(0x1000), Ok(0x42));

        // Test NULL pointer access
        assert!(memory.read_u8(0x0000).is_err());
        assert!(memory.write_u8(0x0000, 0x42).is_err());
    }

    #[test]
    fn test_read_write_u32_little_endian() {
        let mut memory = Memory::new(0x10000);

        // Write 32-bit value
        memory.write_u32(0x1000, 0x12345678).unwrap();

        // Check individual bytes (little-endian)
        assert_eq!(memory.read_u8(0x1000), Ok(0x78));  // LSB first
        assert_eq!(memory.read_u8(0x1001), Ok(0x56));
        assert_eq!(memory.read_u8(0x1002), Ok(0x34));

        // Read back as 32-bit value
        assert_eq!(memory.read_u32(0x1000), Ok(0x12345678));
    }

    #[test]
    fn test_stack_operations() {
        let mut memory = Memory::new(0x1_0000_0000);  // 256MB for stack

        // Test push
        let new_esp = memory.push_u32(0xFFFF0000, 0x12345678).unwrap();
        assert_eq!(new_esp, 0xFFFEFFFC);

        // Test pop
        let (value, final_esp) = memory.pop_u32(new_esp).unwrap();
        assert_eq!(value, 0x12345678);
        assert_eq!(final_esp, 0xFFFF0000);
    }

    #[test]
    fn test_mmio_operations() {
        let mut memory = Memory::new(0x10000);

        // Test out of bounds read
        let mem = Memory::default();
        let addr = mem.size() as u32; // definitely out of bounds
        let err = mem.read_u8(addr).unwrap_err();
        assert!(format!("{}", err).contains("out of bounds"));

        // Test LED write
        memory.write_u8(0xFFFF0004, 0xFF).unwrap();
        assert_eq!(memory.mmio.get_leds(), 0xFF);

        // Test 7-segment write
        memory.write_u8(0xFFFF0008, 0x3F).unwrap();
        assert_eq!(memory.mmio.get_seven_segment(), 0x3F);

        // Test switch read
        memory.mmio.set_switches(0x0A);
        assert_eq!(memory.read_u8(0xFFFF0000), Ok(0x0A));
    }

    #[test]
    fn test_sub_u8_memory() {
        let mut mem = Memory::new(0x2000);
        mem.write_u8(0x1100, 20).unwrap();
        mem.write_u8(0x1101, 5).unwrap();
        let a = mem.read_u8(0x1100).unwrap();
        let b = mem.read_u8(0x1101).unwrap();
        let (result, flags) = crate::instructions::sub::sub8(crate::instructions::sub::CpuFlags::default(), a, b);
        assert_eq!(result, 15);
        assert!(!flags.cf);
        assert!(!flags.zf);
    }

    #[test]
    fn test_sub_u32_memory() {
        let mut mem = Memory::new(0x2000);
        mem.write_u32(0x1200, 100).unwrap();
        mem.write_u32(0x1204, 40).unwrap();
        let a = mem.read_u32(0x1200).unwrap();
        let b = mem.read_u32(0x1204).unwrap();
        let (result, flags) = crate::instructions::sub::sub32(crate::instructions::sub::CpuFlags::default(), a, b);
        assert_eq!(result, 60);
        assert!(!flags.cf);
        assert!(!flags.zf);
    }

    #[test]
    fn test_sub_u8_memory_borrow() {
        let mut mem = Memory::new(0x2000);
        mem.write_u8(0x1102, 5).unwrap();
        mem.write_u8(0x1103, 10).unwrap();
        let a = mem.read_u8(0x1102).unwrap();
        let b = mem.read_u8(0x1103).unwrap();
        let (result, flags) = crate::instructions::sub::sub8(crate::instructions::sub::CpuFlags::default(), a, b);
        assert_eq!(result, 251); // 5 - 10 = 251 (u8 wrap)
        assert!(flags.cf); // borrow occurred
        assert!(!flags.zf);
    }

    #[test]
    fn test_sub_u32_memory_zero() {
        let mut mem = Memory::new(0x2000);
        mem.write_u32(0x1208, 42).unwrap();
        mem.write_u32(0x120C, 42).unwrap();
        let a = mem.read_u32(0x1208).unwrap();
        let b = mem.read_u32(0x120C).unwrap();
        let (result, flags) = crate::instructions::sub::sub32(crate::instructions::sub::CpuFlags::default(), a, b);
        assert_eq!(result, 0);
        assert!(flags.zf); // zero flag set
        assert!(!flags.cf);
    }
}

