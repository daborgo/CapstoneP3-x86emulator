# Implementing the Memory Module

## Overview
This document provides a step-by-step guide to implementing the CPU memory system in Rust. Every line of code is explained with the underlying concepts, addressing modes, byte ordering, and MMIO handling.

## File: `core/src/cpu/memory.rs`

### Step 1: Module Declaration and Imports

```rust
//! CPU Memory Module
//! 
//! This module implements the x86-32 memory system including:
//! - RAM simulation using Vec<u8>
//! - Memory-mapped I/O (MMIO) for devices
//! - Little-endian byte ordering
//! - Bounds checking and security
//! - Stack operations
//! 
//! Key concepts:
//! - Memory is byte-addressable (each byte has unique address)
//! - 32-bit addressing = 4 GB address space
//! - Little-endian: least significant byte first
//! - MMIO: special addresses control devices instead of RAM
//! - Stack grows downward (high to low addresses)

use std::fmt;
use std::error::Error;
```

**Explanation**:
- `//!` creates module-level documentation
- Explains all major components and concepts
- `use std::fmt;` for implementing `Display` trait
- `use std::error::Error;` for error handling

### Step 2: Error Types

```rust
/// Memory access errors
/// 
/// These errors can occur during memory operations and provide
/// detailed information about what went wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryError {
    /// Attempted to access memory outside valid range
    OutOfBounds {
        address: u32,
        size: u32,
        max_address: u32,
    },
    
    /// Attempted to access NULL region (0x0000-0x0FFF)
    NullPointerAccess {
        address: u32,
    },
    
    /// Attempted to access invalid MMIO address
    InvalidMmioAddress {
        address: u32,
    },
    
    /// Stack overflow (ESP went too low)
    StackOverflow {
        esp: u32,
        stack_limit: u32,
    },
    
    /// Stack underflow (ESP went too high)
    StackUnderflow {
        esp: u32,
        stack_base: u32,
    },
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::OutOfBounds { address, size, max_address } => {
                write!(f, "Memory access out of bounds: address 0x{:08X}, size {}, max 0x{:08X}", 
                       address, size, max_address)
            },
            MemoryError::NullPointerAccess { address } => {
                write!(f, "NULL pointer access attempted: address 0x{:08X}", address)
            },
            MemoryError::InvalidMmioAddress { address } => {
                write!(f, "Invalid MMIO address: 0x{:08X}", address)
            },
            MemoryError::StackOverflow { esp, stack_limit } => {
                write!(f, "Stack overflow: ESP 0x{:08X} below limit 0x{:08X}", esp, stack_limit)
            },
            MemoryError::StackUnderflow { esp, stack_base } => {
                write!(f, "Stack underflow: ESP 0x{:08X} above base 0x{:08X}", esp, stack_base)
            },
        }
    }
}

impl Error for MemoryError {}
```

**Error handling design**:

1. **Specific error types**: Each error has detailed context
2. **Display implementation**: Human-readable error messages
3. **Error trait**: Integrates with Rust's error handling
4. **Debug/Clone**: For testing and error propagation

### Step 3: MMIO Device Structure

```rust
/// Memory-Mapped I/O Devices
/// 
/// This struct represents the I/O devices accessible through MMIO.
/// Each device occupies specific memory addresses and responds to reads/writes.
/// 
/// Device map (from project specification):
/// - 0xFFFF0000: Switch register (8-bit read)
/// - 0xFFFF0004: LED register (8-bit write)  
/// - 0xFFFF0008: 7-segment display (8-bit write)
/// - 0xFFFF000C: I/O control register (8-bit read/write)
#[derive(Debug, Clone)]
pub struct IoDevices {
    /// Switch states (8 switches, each bit represents one switch)
    /// Bit 0 = Switch 0, Bit 1 = Switch 1, etc.
    switches: u8,
    
    /// LED states (8 LEDs, each bit represents one LED)
    /// Bit 0 = LED 0, Bit 1 = LED 1, etc.
    leds: u8,
    
    /// 7-segment display value
    /// Encoded as: Bit 0=a, Bit 1=b, Bit 2=c, Bit 3=d, Bit 4=e, Bit 5=f, Bit 6=g, Bit 7=dp
    seven_segment: u8,
    
    /// I/O control register
    /// Used for device control and status
    io_ctrl: u8,
}

impl IoDevices {
    /// Create new I/O devices with default state
    /// 
    /// All devices start in a known state:
    /// - Switches: All off (0x00)
    /// - LEDs: All off (0x00)
    /// - 7-segment: Display '0' (0x3F)
    /// - I/O control: Default state (0x00)
    pub fn new() -> Self {
        IoDevices {
            switches: 0x00,        // All switches off
            leds: 0x00,           // All LEDs off
            seven_segment: 0x3F,   // Display '0' (segments a,b,c,d,e,f)
            io_ctrl: 0x00,         // Default control state
        }
    }
    
    /// Read from MMIO address
    /// 
    /// # Arguments
    /// * `address` - The MMIO address to read from
    /// 
    /// # Returns
    /// The value read from the device, or error if invalid address
    /// 
    /// # Example
    /// ```rust
    /// let mut devices = IoDevices::new();
    /// devices.switches = 0x0A;  // Switches 1 and 3 on
    /// assert_eq!(devices.read(0xFFFF0000), Ok(0x0A));
    /// ```
    pub fn read(&self, address: u32) -> Result<u8, MemoryError> {
        match address {
            0xFFFF0000 => Ok(self.switches),
            0xFFFF0004 => Err(MemoryError::InvalidMmioAddress { address }), // LEDs are write-only
            0xFFFF0008 => Err(MemoryError::InvalidMmioAddress { address }), // 7-seg is write-only
            0xFFFF000C => Ok(self.io_ctrl),
            _ => Err(MemoryError::InvalidMmioAddress { address }),
        }
    }
    
    /// Write to MMIO address
    /// 
    /// # Arguments
    /// * `address` - The MMIO address to write to
    /// * `value` - The value to write
    /// 
    /// # Returns
    /// Ok(()) on success, or error if invalid address
    /// 
    /// # Example
    /// ```rust
    /// let mut devices = IoDevices::new();
    /// devices.write(0xFFFF0004, 0xFF);  // Turn on all LEDs
    /// assert_eq!(devices.leds, 0xFF);
    /// ```
    pub fn write(&mut self, address: u32, value: u8) -> Result<(), MemoryError> {
        match address {
            0xFFFF0000 => Err(MemoryError::InvalidMmioAddress { address }), // Switches are read-only
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
    /// 
    /// # Returns
    /// The current LED state (8 bits)
    pub fn get_leds(&self) -> u8 {
        self.leds
    }
    
    /// Get current 7-segment display value
    /// 
    /// # Returns
    /// The current 7-segment display value
    pub fn get_seven_segment(&self) -> u8 {
        self.seven_segment
    }
    
    /// Set switch states (for testing/simulation)
    /// 
    /// # Arguments
    /// * `switches` - New switch state (8 bits)
    /// 
    /// # Note
    /// In real hardware, switches would be read from physical inputs.
    /// This method is for testing and simulation purposes.
    pub fn set_switches(&mut self, switches: u8) {
        self.switches = switches;
    }
}
```

**MMIO design decisions**:

1. **Device separation**: Each device has its own field
2. **Read/write restrictions**: Some devices are read-only or write-only
3. **Error handling**: Invalid addresses return specific errors
4. **Testing support**: Methods to simulate device state changes

### Step 4: Memory Structure Definition

```rust
/// CPU Memory Structure
/// 
/// Represents the complete memory system including:
/// - RAM (main memory)
/// - MMIO devices (LEDs, switches, 7-segment display)
/// - Memory layout and addressing
/// 
/// Memory layout:
/// - 0x00000000-0x00000FFF: Reserved (NULL region)
/// - 0x00001000-0x000FFFFF: Code section
/// - 0x00100000-0x001FFFFF: Data section  
/// - 0x00200000-0xFFFEFFFF: Heap (grows upward)
/// - 0xFFFF0000-0xFFFFFFFF: Stack (grows downward) + MMIO
/// 
/// Design choice: Vec<u8> vs [u8; SIZE]
/// - Vec<u8>: Flexible size, heap-allocated, can be resized
/// - [u8; SIZE]: Fixed size, stack-allocated, faster access
/// 
/// We choose Vec<u8> for flexibility and to avoid stack overflow.
#[derive(Debug, Clone)]
pub struct Memory {
    /// Main RAM - vector of bytes
    /// Index represents memory address
    ram: Vec<u8>,
    
    /// Memory-mapped I/O devices
    mmio: IoDevices,
    
    /// Memory size in bytes
    size: usize,
    
    /// Stack base address (where stack starts)
    stack_base: u32,
    
    /// Stack limit (lowest valid stack address)
    stack_limit: u32,
}
```

**Memory structure design**:

1. **`ram: Vec<u8>`**: Main memory as byte array
2. **`mmio: IoDevices`**: Separate MMIO device handling
3. **`size: usize`**: Total memory size for bounds checking
4. **`stack_base/limit`**: Stack boundary tracking

### Step 5: Constructor Implementation

```rust
impl Memory {
    /// Create new memory with specified size
    /// 
    /// # Arguments
    /// * `size` - Total memory size in bytes
    /// 
    /// # Returns
    /// A new Memory instance with all bytes initialized to 0
    /// 
    /// # Example
    /// ```rust
    /// let memory = Memory::new(16 * 1024 * 1024);  // 16 MB
    /// ```
    pub fn new(size: usize) -> Self {
        // Validate size
        if size == 0 {
            panic!("Memory size must be greater than 0");
        }
        if size > 0x1_0000_0000 {
            panic!("Memory size cannot exceed 4 GB");
        }
        
        Memory {
            ram: vec![0; size],           // Initialize all bytes to 0
            mmio: IoDevices::new(),       // Initialize MMIO devices
            size,                         // Store size for bounds checking
            stack_base: 0xFFFF_0000,      // Stack starts at high address
            stack_limit: 0xFF00_0000,     // Stack limit (16 MB below base)
        }
    }
    
    /// Create memory with default size (16 MB)
    /// 
    /// This is a convenience method for common use cases.
    /// 16 MB is sufficient for most emulation tasks.
    /// 
    /// # Returns
    /// A new Memory instance with 16 MB of RAM
    /// 
    /// # Example
    /// ```rust
    /// let memory = Memory::default();
    /// ```
    pub fn default() -> Self {
        Self::new(16 * 1024 * 1024)  // 16 MB
    }
    
    /// Get memory size
    /// 
    /// # Returns
    /// The total memory size in bytes
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Get stack base address
    /// 
    /// # Returns
    /// The stack base address
    pub fn stack_base(&self) -> u32 {
        self.stack_base
    }
    
    /// Get stack limit address
    /// 
    /// # Returns
    /// The stack limit address
    pub fn stack_limit(&self) -> u32 {
        self.stack_limit
    }
}
```

**Constructor explanations**:

1. **`new(size)`**: Create memory with specific size
2. **`default()`**: Create memory with 16 MB (convenience)
3. **Validation**: Check size is reasonable
4. **Initialization**: All bytes start at 0
5. **Stack setup**: Define stack boundaries

### Step 6: Address Validation Methods

```rust
impl Memory {
    /// Check if address is in MMIO region
    /// 
    /// # Arguments
    /// * `address` - The address to check
    /// 
    /// # Returns
    /// True if address is in MMIO region (0xFFFF0000-0xFFFFFFFF)
    fn is_mmio_address(&self, address: u32) -> bool {
        address >= 0xFFFF_0000
    }
    
    /// Check if address is in NULL region
    /// 
    /// # Arguments
    /// * `address` - The address to check
    /// 
    /// # Returns
    /// True if address is in NULL region (0x0000-0x0FFF)
    fn is_null_address(&self, address: u32) -> bool {
        address < 0x1000
    }
    
    /// Check if address is in valid RAM range
    /// 
    /// # Arguments
    /// * `address` - The address to check
    /// 
    /// # Returns
    /// True if address is in valid RAM range
    fn is_valid_ram_address(&self, address: u32) -> bool {
        address < self.size as u32
    }
    
    /// Validate address and size for memory access
    /// 
    /// # Arguments
    /// * `address` - The starting address
    /// * `size` - The number of bytes to access
    /// 
    /// # Returns
    /// Ok(()) if valid, or appropriate error
    fn validate_access(&self, address: u32, size: u32) -> Result<(), MemoryError> {
        // Check for NULL pointer access
        if self.is_null_address(address) {
            return Err(MemoryError::NullPointerAccess { address });
        }
        
        // Check for MMIO access (handled separately)
        if self.is_mmio_address(address) {
            return Ok(());  // MMIO validation handled in MMIO methods
        }
        
        // Check bounds for RAM access
        let end_address = address.wrapping_add(size - 1);
        if !self.is_valid_ram_address(end_address) {
            return Err(MemoryError::OutOfBounds {
                address,
                size,
                max_address: self.size as u32 - 1,
            });
        }
        
        Ok(())
    }
}
```

**Address validation design**:

1. **`is_mmio_address()`**: Check if address is in MMIO region
2. **`is_null_address()`**: Check if address is in NULL region
3. **`is_valid_ram_address()`**: Check if address is in RAM
4. **`validate_access()`**: Comprehensive validation with error details

### Step 7: Read Operations

```rust
impl Memory {
    /// Read a single byte from memory
    /// 
    /// # Arguments
    /// * `address` - The address to read from
    /// 
    /// # Returns
    /// The byte value, or error if invalid access
    /// 
    /// # Example
    /// ```rust
    /// let mut memory = Memory::default();
    /// memory.write_u8(0x1000, 0x42).unwrap();
    /// assert_eq!(memory.read_u8(0x1000), Ok(0x42));
    /// ```
    pub fn read_u8(&self, address: u32) -> Result<u8, MemoryError> {
        // Validate access
        self.validate_access(address, 1)?;
        
        // Handle MMIO
        if self.is_mmio_address(address) {
            return self.mmio.read(address);
        }
        
        // Read from RAM
        Ok(self.ram[address as usize])
    }
    
    /// Read a 16-bit word from memory (little-endian)
    /// 
    /// # Arguments
    /// * `address` - The address to read from
    /// 
    /// # Returns
    /// The 16-bit value, or error if invalid access
    /// 
    /// # Example
    /// ```rust
    /// let mut memory = Memory::default();
    /// memory.write_u16(0x1000, 0x1234).unwrap();
    /// assert_eq!(memory.read_u16(0x1000), Ok(0x1234));
    /// ```
    pub fn read_u16(&self, address: u32) -> Result<u16, MemoryError> {
        // Validate access
        self.validate_access(address, 2)?;
        
        // Handle MMIO (not supported for multi-byte reads)
        if self.is_mmio_address(address) {
            return Err(MemoryError::InvalidMmioAddress { address });
        }
        
        // Read bytes in little-endian order
        let byte0 = self.ram[address as usize] as u16;
        let byte1 = self.ram[(address + 1) as usize] as u16;
        
        // Combine bytes: byte1 is MSB, byte0 is LSB
        Ok((byte1 << 8) | byte0)
    }
    
    /// Read a 32-bit double word from memory (little-endian)
    /// 
    /// # Arguments
    /// * `address` - The address to read from
    /// 
    /// # Returns
    /// The 32-bit value, or error if invalid access
    /// 
    /// # Example
    /// ```rust
    /// let mut memory = Memory::default();
    /// memory.write_u32(0x1000, 0x12345678).unwrap();
    /// assert_eq!(memory.read_u32(0x1000), Ok(0x12345678));
    /// ```
    pub fn read_u32(&self, address: u32) -> Result<u32, MemoryError> {
        // Validate access
        self.validate_access(address, 4)?;
        
        // Handle MMIO (not supported for multi-byte reads)
        if self.is_mmio_address(address) {
            return Err(MemoryError::InvalidMmioAddress { address });
        }
        
        // Read bytes in little-endian order
        let byte0 = self.ram[address as usize] as u32;
        let byte1 = self.ram[(address + 1) as usize] as u32;
        let byte2 = self.ram[(address + 2) as usize] as u32;
        let byte3 = self.ram[(address + 3) as usize] as u32;
        
        // Combine bytes: byte3 is MSB, byte0 is LSB
        Ok((byte3 << 24) | (byte2 << 16) | (byte1 << 8) | byte0)
    }
}
```

**Read operation details**:

1. **Validation**: Check address and size before access
2. **MMIO handling**: Special case for device addresses
3. **Little-endian**: LSB first, MSB last
4. **Error propagation**: Use `?` operator for clean error handling

### Step 8: Write Operations

```rust
impl Memory {
    /// Write a single byte to memory
    /// 
    /// # Arguments
    /// * `address` - The address to write to
    /// * `value` - The byte value to write
    /// 
    /// # Returns
    /// Ok(()) on success, or error if invalid access
    /// 
    /// # Example
    /// ```rust
    /// let mut memory = Memory::default();
    /// memory.write_u8(0x1000, 0x42).unwrap();
    /// assert_eq!(memory.read_u8(0x1000), Ok(0x42));
    /// ```
    pub fn write_u8(&mut self, address: u32, value: u8) -> Result<(), MemoryError> {
        // Validate access
        self.validate_access(address, 1)?;
        
        // Handle MMIO
        if self.is_mmio_address(address) {
            return self.mmio.write(address, value);
        }
        
        // Write to RAM
        self.ram[address as usize] = value;
        Ok(())
    }
    
    /// Write a 16-bit word to memory (little-endian)
    /// 
    /// # Arguments
    /// * `address` - The address to write to
    /// * `value` - The 16-bit value to write
    /// 
    /// # Returns
    /// Ok(()) on success, or error if invalid access
    /// 
    /// # Example
    /// ```rust
    /// let mut memory = Memory::default();
    /// memory.write_u16(0x1000, 0x1234).unwrap();
    /// assert_eq!(memory.read_u16(0x1000), Ok(0x1234));
    /// ```
    pub fn write_u16(&mut self, address: u32, value: u16) -> Result<(), MemoryError> {
        // Validate access
        self.validate_access(address, 2)?;
        
        // Handle MMIO (not supported for multi-byte writes)
        if self.is_mmio_address(address) {
            return Err(MemoryError::InvalidMmioAddress { address });
        }
        
        // Write bytes in little-endian order
        self.ram[address as usize] = (value & 0xFF) as u8;        // LSB first
        self.ram[(address + 1) as usize] = ((value >> 8) & 0xFF) as u8;  // MSB second
        
        Ok(())
    }
    
    /// Write a 32-bit double word to memory (little-endian)
    /// 
    /// # Arguments
    /// * `address` - The address to write to
    /// * `value` - The 32-bit value to write
    /// 
    /// # Returns
    /// Ok(()) on success, or error if invalid access
    /// 
    /// # Example
    /// ```rust
    /// let mut memory = Memory::default();
    /// memory.write_u32(0x1000, 0x12345678).unwrap();
    /// assert_eq!(memory.read_u32(0x1000), Ok(0x12345678));
    /// ```
    pub fn write_u32(&mut self, address: u32, value: u32) -> Result<(), MemoryError> {
        // Validate access
        self.validate_access(address, 4)?;
        
        // Handle MMIO (not supported for multi-byte writes)
        if self.is_mmio_address(address) {
            return Err(MemoryError::InvalidMmioAddress { address });
        }
        
        // Write bytes in little-endian order
        self.ram[address as usize] = (value & 0xFF) as u8;           // Byte 0 (LSB)
        self.ram[(address + 1) as usize] = ((value >> 8) & 0xFF) as u8;   // Byte 1
        self.ram[(address + 2) as usize] = ((value >> 16) & 0xFF) as u8;   // Byte 2
        self.ram[(address + 3) as usize] = ((value >> 24) & 0xFF) as u8;   // Byte 3 (MSB)
        
        Ok(())
    }
}
```

**Write operation details**:

1. **Validation**: Check address and size before access
2. **MMIO handling**: Special case for device addresses
3. **Little-endian**: LSB first, MSB last
4. **Bit masking**: Extract individual bytes with `& 0xFF`

### Step 9: Stack Operations

```rust
impl Memory {
    /// Push a 32-bit value onto the stack
    /// 
    /// # Arguments
    /// * `esp` - Current stack pointer (will be updated)
    /// * `value` - The value to push
    /// 
    /// # Returns
    /// New stack pointer value, or error if stack overflow
    /// 
    /// # Example
    /// ```rust
    /// let mut memory = Memory::default();
    /// let new_esp = memory.push_u32(0xFFFF0000, 0x12345678).unwrap();
    /// assert_eq!(new_esp, 0xFFFEFFFC);  // ESP moved down 4 bytes
    /// ```
    pub fn push_u32(&mut self, esp: u32, value: u32) -> Result<u32, MemoryError> {
        // Check for stack overflow
        let new_esp = esp.wrapping_sub(4);
        if new_esp < self.stack_limit {
            return Err(MemoryError::StackOverflow {
                esp: new_esp,
                stack_limit: self.stack_limit,
            });
        }
        
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
    /// Tuple of (value, new_stack_pointer), or error if stack underflow
    /// 
    /// # Example
    /// ```rust
    /// let mut memory = Memory::default();
    /// memory.write_u32(0xFFFEFFFC, 0x12345678).unwrap();
    /// let (value, new_esp) = memory.pop_u32(0xFFFEFFFC).unwrap();
    /// assert_eq!(value, 0x12345678);
    /// assert_eq!(new_esp, 0xFFFF0000);
    /// ```
    pub fn pop_u32(&mut self, esp: u32) -> Result<(u32, u32), MemoryError> {
        // Check for stack underflow
        if esp >= self.stack_base {
            return Err(MemoryError::StackUnderflow {
                esp,
                stack_base: self.stack_base,
            });
        }
        
        // Read value from stack
        let value = self.read_u32(esp)?;
        
        // Calculate new stack pointer
        let new_esp = esp.wrapping_add(4);
        
        Ok((value, new_esp))
    }
    
    /// Check if stack pointer is valid
    /// 
    /// # Arguments
    /// * `esp` - The stack pointer to check
    /// 
    /// # Returns
    /// True if stack pointer is within valid range
    pub fn is_valid_stack_pointer(&self, esp: u32) -> bool {
        esp >= self.stack_limit && esp < self.stack_base
    }
}
```

**Stack operation details**:

1. **`push_u32()`**: Decrement ESP, write value, check overflow
2. **`pop_u32()`**: Read value, increment ESP, check underflow
3. **`is_valid_stack_pointer()`**: Check if ESP is in valid range
4. **Wrapping arithmetic**: Use `wrapping_sub/add` for overflow handling

### Step 10: MMIO Access Methods

```rust
impl Memory {
    /// Get MMIO device reference for direct access
    /// 
    /// # Returns
    /// Mutable reference to MMIO devices
    /// 
    /// # Example
    /// ```rust
    /// let mut memory = Memory::default();
    /// memory.get_mmio().set_switches(0x0A);  // Set switches 1 and 3
    /// ```
    pub fn get_mmio(&mut self) -> &mut IoDevices {
        &mut self.mmio
    }
    
    /// Get MMIO device reference for read-only access
    /// 
    /// # Returns
    /// Immutable reference to MMIO devices
    pub fn get_mmio_ref(&self) -> &IoDevices {
        &self.mmio
    }
}
```

**MMIO access design**:

1. **`get_mmio()`**: Mutable access for device control
2. **`get_mmio_ref()`**: Immutable access for reading state
3. **Encapsulation**: MMIO devices are private, accessed through methods

### Step 11: Display Implementation

```rust
impl fmt::Display for Memory {
    /// Format memory for human-readable output
    /// 
    /// Shows memory size, stack boundaries, and MMIO device states.
    /// 
    /// # Example
    /// ```
    /// let memory = Memory::default();
    /// println!("{}", memory);
    /// // Output:
    /// // Memory: 16 MB
    /// // Stack: 0xFF000000 - 0xFFFF0000
    /// // MMIO: LEDs=0x00, 7-seg=0x3F, Switches=0x00
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Memory: {} MB", self.size / (1024 * 1024))?;
        writeln!(f, "Stack: 0x{:08X} - 0x{:08X}", self.stack_limit, self.stack_base)?;
        writeln!(f, "MMIO: LEDs=0x{:02X}, 7-seg=0x{:02X}, Switches=0x{:02X}", 
                 self.mmio.get_leds(), 
                 self.mmio.get_seven_segment(),
                 self.mmio.switches)?;
        Ok(())
    }
}
```

### Step 12: Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(1024);  // 1 KB for testing
        
        assert_eq!(memory.size(), 1024);
        assert_eq!(memory.stack_base(), 0xFFFF_0000);
        assert_eq!(memory.stack_limit(), 0xFF00_0000);
    }
    
    #[test]
    fn test_read_write_u8() {
        let mut memory = Memory::new(1024);
        
        // Test normal write/read
        memory.write_u8(0x1000, 0x42).unwrap();
        assert_eq!(memory.read_u8(0x1000), Ok(0x42));
        
        // Test NULL pointer access
        assert!(memory.read_u8(0x0000).is_err());
        assert!(memory.write_u8(0x0000, 0x42).is_err());
        
        // Test out of bounds
        assert!(memory.read_u8(0x1000).is_err());  // Beyond 1KB
    }
    
    #[test]
    fn test_read_write_u32_little_endian() {
        let mut memory = Memory::new(1024);
        
        // Write 32-bit value
        memory.write_u32(0x1000, 0x12345678).unwrap();
        
        // Check individual bytes (little-endian)
        assert_eq!(memory.read_u8(0x1000), Ok(0x78));  // LSB first
        assert_eq!(memory.read_u8(0x1001), Ok(0x56));
        assert_eq!(memory.read_u8(0x1002), Ok(0x34));
        assert_eq!(memory.read_u8(0x1003), Ok(0x12));  // MSB last
        
        // Read back as 32-bit value
        assert_eq!(memory.read_u32(0x1000), Ok(0x12345678));
    }
    
    #[test]
    fn test_stack_operations() {
        let mut memory = Memory::new(1024);
        
        // Test push
        let new_esp = memory.push_u32(0xFFFF0000, 0x12345678).unwrap();
        assert_eq!(new_esp, 0xFFFEFFFC);
        
        // Test pop
        let (value, final_esp) = memory.pop_u32(new_esp).unwrap();
        assert_eq!(value, 0x12345678);
        assert_eq!(final_esp, 0xFFFF0000);
        
        // Test stack overflow
        let result = memory.push_u32(0xFF000000, 0x42);
        assert!(result.is_err());
        
        // Test stack underflow
        let result = memory.pop_u32(0xFFFF0000);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_mmio_operations() {
        let mut memory = Memory::new(1024);
        
        // Test LED write
        memory.write_u8(0xFFFF0004, 0xFF).unwrap();
        assert_eq!(memory.get_mmio_ref().get_leds(), 0xFF);
        
        // Test 7-segment write
        memory.write_u8(0xFFFF0008, 0x3F).unwrap();
        assert_eq!(memory.get_mmio_ref().get_seven_segment(), 0x3F);
        
        // Test switch read
        memory.get_mmio().set_switches(0x0A);
        assert_eq!(memory.read_u8(0xFFFF0000), Ok(0x0A));
        
        // Test write-only restrictions
        assert!(memory.write_u8(0xFFFF0000, 0x42).is_err());  // Switches are read-only
        assert!(memory.read_u8(0xFFFF0004).is_err());         // LEDs are write-only
    }
    
    #[test]
    fn test_validation() {
        let memory = Memory::new(1024);
        
        // Test valid addresses
        assert!(memory.is_valid_ram_address(0x1000));
        assert!(memory.is_mmio_address(0xFFFF0000));
        assert!(memory.is_null_address(0x0000));
        
        // Test invalid addresses
        assert!(!memory.is_valid_ram_address(0x1000));  // Beyond 1KB
        assert!(!memory.is_mmio_address(0x1000));
        assert!(!memory.is_null_address(0x1000));
    }
}
```

**Test explanations**:

1. **`test_memory_creation()`**: Test constructor and default values
2. **`test_read_write_u8()`**: Test basic byte operations and error handling
3. **`test_read_write_u32_little_endian()`**: Test multi-byte operations and byte ordering
4. **`test_stack_operations()`**: Test push/pop and stack boundary checking
5. **`test_mmio_operations()`**: Test MMIO device access and restrictions
6. **`test_validation()`**: Test address validation methods

## Summary

This implementation provides:

1. **Complete memory system** with RAM and MMIO support
2. **Little-endian byte ordering** matching x86 architecture
3. **Comprehensive error handling** with detailed error types
4. **Stack operations** with overflow/underflow detection
5. **MMIO device simulation** for LEDs, switches, and 7-segment display
6. **Bounds checking** for security and debugging
7. **Extensive testing** for all functionality

The memory module is now ready to be integrated with the registers and flags modules to form the complete CPU state.
