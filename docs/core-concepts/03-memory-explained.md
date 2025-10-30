# Understanding Memory in CPU Emulation

## Table of Contents
1. [What is Memory?](#what-is-memory)
2. [Memory Addressing](#memory-addressing)
3. [Byte Ordering: Little-Endian vs Big-Endian](#byte-ordering-little-endian-vs-big-endian)
4. [Memory Layout](#memory-layout)
5. [The Stack](#the-stack)
6. [Memory-Mapped I/O (MMIO)](#memory-mapped-io-mmio)
7. [Memory Operations](#memory-operations)
8. [Bounds Checking and Security](#bounds-checking-and-security)
9. [Rust Implementation with Vec<u8>](#rust-implementation-with-vecu8)

---

## What is Memory?

### Simple Definition
**Memory (RAM) is a large array of bytes where the CPU stores both programs (instructions) and data (variables).** It's like a giant filing cabinet where each drawer has a unique address.

### The Analogy
Think of memory like an apartment building:
- **Each byte** = An apartment
- **Address** = The apartment number
- **Contents** = Who lives there (the data value)
- **CPU** = The mail carrier who delivers and picks up mail (reads/writes data)

Just like you need an apartment number to deliver mail, the CPU needs a memory address to read or write data.

### Key Characteristics
- **Byte-Addressable**: Each byte has a unique address
- **Random Access**: Can access any byte directly (hence RAM - Random Access Memory)
- **Volatile**: In real hardware, powered off = data lost (we'll persist ours in memory/files)
- **Large**: 32-bit addressing = 4 GB of addressable space
- **Slower than Registers**: ~100x slower, but much more spacious

---

## Memory Addressing

### What is an Address?

An **address** is a number that identifies a specific byte in memory.

```
Memory Address    Value (1 byte)    What might be stored there
0x00000000          0x55            Code or data
0x00000001          0x89            Code or data
0x00000002          0xE5            Code or data
...
0x00001000          0x42            Perhaps the number 66
0x00001001          0x00            
0x00001002          0x00
0x00001003          0x00
...
0xFFFF0004          0xFF            LED register (I/O)
0xFFFFFFFF          0x00            Last byte in 4GB space
```

### Address Space

In 32-bit x86, addresses are 32 bits wide:
- **Minimum address**: `0x00000000` (0)
- **Maximum address**: `0xFFFFFFFF` (4,294,967,295)
- **Total addressable**: 2^32 bytes = 4 GB

```
32-bit address:
┌───────────────────────────────────────┐
│ 00000000 00000000 00010000 00000000  │ = 0x00001000
└───────────────────────────────────────┘
  Byte 3   Byte 2   Byte 1   Byte 0

This address points to byte #4096 in memory
```

### Multi-Byte Values

Most values are larger than 1 byte:
- **Byte (8 bits)**: 1 address
- **Word (16 bits)**: 2 consecutive addresses
- **Dword (32 bits)**: 4 consecutive addresses (most common in 32-bit)

**Example: Storing a 32-bit value**
```
Value to store: 0x12345678

Memory layout:
Address     Value
0x1000:     0x78  ← Byte 0 (least significant)
0x1001:     0x56  ← Byte 1
0x1002:     0x34  ← Byte 2
0x1003:     0x12  ← Byte 3 (most significant)
```

**Key point**: The value `0x12345678` spans 4 bytes starting at address `0x1000`.

---

## Byte Ordering: Little-Endian vs Big-Endian

### The Problem

When storing multi-byte values, which byte goes first?

**Example**: Store the 32-bit number `0x12345678`
- Byte 0 (least significant): `0x78`
- Byte 1: `0x56`
- Byte 2: `0x34`
- Byte 3 (most significant): `0x12`

### Little-Endian (x86 uses this!)

**"Little end first"** - Store the **least significant byte** at the **lowest address**

```
Value: 0x12345678

Address:  0x1000  0x1001  0x1002  0x1003
          ┌────┬─────┬─────┬─────┐
Memory:   │0x78│0x56 │0x34 │0x12 │
          └────┴─────┴─────┴─────┘
            ↑                  ↑
          LSB                MSB
         (little end)      (big end)
```

**Think**: The "little end" (least significant byte) comes first.

### Big-Endian (Network byte order)

**"Big end first"** - Store the **most significant byte** at the **lowest address**

```
Value: 0x12345678

Address:  0x1000  0x1001  0x1002  0x1003
          ┌────┬─────┬─────┬─────┐
Memory:   │0x12│0x34 │0x56 │0x78 │
          └────┴─────┴─────┴─────┘
            ↑                  ↑
          MSB                LSB
         (big end)        (little end)
```

**Think**: Matches how we write numbers (most significant digit first).

### Why Does x86 Use Little-Endian?

Historical reasons from the Intel 8080 processor. **For our emulator, we must use little-endian to match x86 behavior.**

### Little-Endian Example

**Store `0xAABBCCDD` at address `0x2000`**:
```
Address     Value       Explanation
0x2000:     0xDD        Least significant byte (rightmost)
0x2001:     0xCC
0x2002:     0xBB
0x2003:     0xAA        Most significant byte (leftmost)
```

**Read 32-bit value from address `0x2000`**:
```rust
let byte0 = memory[0x2000];  // 0xDD
let byte1 = memory[0x2001];  // 0xCC
let byte2 = memory[0x2002];  // 0xBB
let byte3 = memory[0x2003];  // 0xAA

let value = (byte3 << 24) | (byte2 << 16) | (byte1 << 8) | byte0;
// value = 0xAABBCCDD
```

**Key Point**: Bytes are stored "backwards" from how we write hex numbers!

---

## Memory Layout

### Our Emulator's Memory Map

We'll divide the 4 GB address space into regions:

```
0x00000000  ┌──────────────────────────┐
            │     Reserved (NULL)       │  0-4KB: Catch null pointer bugs
0x00001000  ├──────────────────────────┤
            │     Code Section          │  Program instructions
0x00100000  ├──────────────────────────┤
            │     Data Section          │  Global variables, constants
0x00200000  ├──────────────────────────┤
            │     Heap                  │  Dynamic allocation (malloc, new)
            │         ↓                 │  Grows upward
            │                           │
            │         (free space)      │
            │                           │
            │         ↑                 │
            │     Stack                 │  Function calls, local variables
0xFFFF0000  ├──────────────────────────┤  Grows downward
            │     MMIO (I/O)            │  Memory-mapped devices
0xFFFFFFFF  └──────────────────────────┘  LEDs, switches, 7-segment

            Total: 4,294,967,296 bytes (4 GB)
```

### Memory Regions Explained

#### 1. Reserved / NULL Region (0x00000000 - 0x00000FFF)
**Purpose**: Catch null pointer dereferences

**Why**: If you accidentally access address `0x00000000` (null pointer), we can detect it and raise an error instead of silently corrupting data.

#### 2. Code Section (0x00001000 - 0x000FFFFF)
**Purpose**: Store program instructions

**Example**:
```
Address      Bytes          Instruction
0x00001000:  01 D8          ADD EAX, EBX
0x00001002:  89 C3          MOV EBX, EAX
0x00001004:  C3             RET
```

The **EIP** (instruction pointer) register points into this region.

#### 3. Data Section (0x00100000 - 0x001FFFFF)
**Purpose**: Store global variables and constants

**Example**:
```c
int global_counter = 42;  // Stored at 0x00100000
const char* message = "Hello";  // Pointer at 0x00100004, string at 0x00100008
```

#### 4. Heap (0x00200000 - grows upward)
**Purpose**: Dynamic memory allocation

**Example**:
```c
int* ptr = malloc(100 * sizeof(int));  // Allocates 400 bytes on heap
// ptr might point to 0x00200000
```

**Grows upward**: Each allocation increases the "heap pointer"

#### 5. Stack (0xFFFF0000 - grows downward)
**Purpose**: Function calls, local variables, return addresses

**Example**:
```c
void func(int a, int b) {
    int local = a + b;  // 'local' stored on stack
}
```

**Grows downward**: Each function call and local variable decreases ESP

#### 6. MMIO Region (0xFFFF0000 - 0xFFFFFFFF)
**Purpose**: Memory-mapped I/O devices

**Example**:
```
Address         Device
0xFFFF0000:     Switch register (read)
0xFFFF0004:     LED register (write)
0xFFFF0008:     7-segment display (write)
```

More on MMIO below!

---

## The Stack

### What is the Stack?

The **stack** is a region of memory used for:
1. **Function calls**: Storing return addresses
2. **Local variables**: Temporary variables in functions
3. **Function parameters**: Arguments passed to functions
4. **Saving registers**: Preserving values across calls

### Stack Characteristics

- **LIFO**: Last In, First Out (like a stack of plates)
- **Grows downward**: In x86, stack grows from high addresses to low
- **ESP points to top**: ESP (stack pointer) always points to the topmost item
- **EBP marks frames**: EBP (base pointer) points to the base of current function's stack frame

### Stack Operations

#### PUSH (Add to stack)
```assembly
PUSH EAX

What happens:
1. ESP = ESP - 4        (Move stack pointer down 4 bytes)
2. [ESP] = EAX          (Write EAX value to top of stack)
```

**Visual**:
```
Before PUSH EAX (EAX = 0x12345678):

0xFFFEFFFC: [empty]
0xFFFF0000: [empty]     ← ESP points here
            ─────────
            Stack Top

After PUSH EAX:

0xFFFEFFFC: 0x12345678  ← ESP now points here (moved down 4 bytes)
0xFFFF0000: [empty]
            ─────────
            Stack Top (grew downward!)
```

#### POP (Remove from stack)
```assembly
POP EBX

What happens:
1. EBX = [ESP]          (Read value from top of stack)
2. ESP = ESP + 4        (Move stack pointer up 4 bytes)
```

**Visual**:
```
Before POP EBX:

0xFFFEFFFC: 0x12345678  ← ESP points here
0xFFFF0000: [empty]
            ─────────

After POP EBX (EBX now = 0x12345678):

0xFFFEFFFC: 0x12345678  (data still there, but no longer "on stack")
0xFFFF0000: [empty]     ← ESP now points here (moved up 4 bytes)
            ─────────
            Stack Top (shrunk upward!)
```

### Stack Frame Example

**C code**:
```c
int add(int a, int b) {
    int result = a + b;
    return result;
}

int main() {
    int x = add(5, 3);
}
```

**Assembly & Stack**:
```assembly
main:
    ; Push parameters (right to left)
    PUSH 3          ; b = 3
    PUSH 5          ; a = 5
    CALL add        ; Push return address, jump to 'add'
    ADD ESP, 8      ; Clean up parameters (2 * 4 bytes)
    ; x is now in EAX
    
add:
    PUSH EBP        ; Save old base pointer
    MOV EBP, ESP    ; Set up new base pointer
    SUB ESP, 4      ; Allocate space for local variable 'result'
    
    ; Function body
    MOV EAX, [EBP+8]   ; Get parameter 'a'
    ADD EAX, [EBP+12]  ; Add parameter 'b'
    MOV [EBP-4], EAX   ; Store in local 'result'
    
    ; Return
    MOV EAX, [EBP-4]   ; Return value in EAX
    MOV ESP, EBP       ; Restore stack pointer
    POP EBP            ; Restore base pointer
    RET                ; Pop return address into EIP
```

**Stack layout during `add` function**:
```
Higher addresses
    0xFFFF0000: [caller's data]
                ...
    0xFFFEFFF8: 3           ← [EBP+12] = parameter b
    0xFFFEFFF4: 5           ← [EBP+8]  = parameter a
    0xFFFEFFF0: 0x00001234  ← [EBP+4]  = return address (pushed by CALL)
    0xFFFEFFEC: 0xFFFF0000  ← [EBP]    = old EBP (pushed by PUSH EBP)
                                        ← EBP points here
    0xFFFEFFE8: 8           ← [EBP-4]  = local variable 'result'
                                        ← ESP points here
Lower addresses
```

**Key insight**: We can access parameters at positive offsets from EBP (`[EBP+8]`), and local variables at negative offsets (`[EBP-4]`).

---

## Memory-Mapped I/O (MMIO)

### What is MMIO?

**Memory-Mapped I/O** means using memory addresses to communicate with hardware devices. **Reading or writing to special addresses controls devices instead of accessing RAM.**

### Why MMIO?

Instead of having separate instructions for I/O:
```assembly
; Hypothetical separate I/O instructions
OUT LED_PORT, EAX     ; Special instruction for I/O
IN EAX, SWITCH_PORT   ; Different from memory access
```

We use regular memory instructions:
```assembly
; MMIO approach
MOV [0xFFFF0004], EAX    ; Write to LED register (same as normal memory!)
MOV EAX, [0xFFFF0000]    ; Read from switch register
```

**Advantage**: Simpler CPU design, no special I/O instructions needed!

### Our MMIO Map (From Project Spec)

```
Base address: IO_BASE = 0xFFFF0000

Device                Address             Access    Description
────────────────────────────────────────────────────────────────
Switches (8-bit)      0xFFFF0000 (SW_REG)     R     Read switch states
LEDs (8-bit)          0xFFFF0004 (LED_REG)    W     Write LED states
7-Segment (8-bit)     0xFFFF0008 (SEG_REG)    W     Write 7-seg display
I/O Control           0xFFFF000C (IO_CTRL)   R/W    Control register (optional)
```

### MMIO Example: Controlling LEDs

**Turn on all 8 LEDs**:
```assembly
MOV AL, 0xFF           ; AL = 11111111 (all bits set)
MOV [0xFFFF0004], AL   ; Write to LED register
```

**What happens**:
1. CPU executes `MOV [0xFFFF0004], AL`
2. Memory system sees address `0xFFFF0004` is in MMIO range
3. Instead of writing to RAM, it triggers LED controller
4. LEDs physically light up (or in our web app, UI updates)

### MMIO Example: Reading Switches

**Read switch states**:
```assembly
MOV AL, [0xFFFF0000]   ; Read from switch register
; AL now contains switch states (e.g., 0b00001010 = switches 1 and 3 are on)
```

### 7-Segment Display Encoding

The 7-segment display has 7 segments (a-g) plus decimal point:

```
     aaaa
    f    b
    f    b
     gggg
    e    c
    e    c
     dddd   (dp)
```

**Bit mapping**:
```
Bit 0 = segment a
Bit 1 = segment b
Bit 2 = segment c
Bit 3 = segment d
Bit 4 = segment e
Bit 5 = segment f
Bit 6 = segment g
Bit 7 = decimal point

Example: Display "0"
 aaaa      Segments: a, b, c, d, e, f (not g)
f    b     Binary: 0b00111111 = 0x3F
e    c
 dddd
```

**Hex digit encoding table**:
```
Digit   Segments        Binary      Hex
0       abcdef          00111111    0x3F
1       bc              00000110    0x06
2       abdeg           01011011    0x5B
3       abcdg           01001111    0x4F
4       bcfg            01100110    0x66
5       acdfg           01101101    0x6D
6       acdefg          01111101    0x7D
7       abc             00000111    0x07
8       abcdefg         01111111    0x7F
9       abcdfg          01101111    0x6F
A       abcefg          01110111    0x77
b       cdefg           01111100    0x7C
C       adef            00111001    0x39
d       bcdeg           01011110    0x5E
E       adefg           01111001    0x79
F       aefg            01110001    0x71
```

**Example: Display "5" on 7-segment**:
```assembly
MOV AL, 0x6D              ; Encoding for '5'
MOV [0xFFFF0008], AL      ; Write to 7-segment register
```

---

## Memory Operations

### Read Operations

#### Read Byte (8-bit)
```rust
fn read_u8(&self, address: u32) -> u8 {
    self.memory[address as usize]
}
```

**Example**:
```
Address 0x1000 contains: 0x42
read_u8(0x1000) returns: 0x42
```

#### Read Word (16-bit, little-endian)
```rust
fn read_u16(&self, address: u32) -> u16 {
    let byte0 = self.read_u8(address) as u16;
    let byte1 = self.read_u8(address + 1) as u16;
    (byte1 << 8) | byte0  // Little-endian: byte0 is LSB
}
```

**Example**:
```
Address 0x1000: 0x34
Address 0x1001: 0x12

read_u16(0x1000) returns: 0x1234
```

#### Read Dword (32-bit, little-endian)
```rust
fn read_u32(&self, address: u32) -> u32 {
    let byte0 = self.read_u8(address) as u32;
    let byte1 = self.read_u8(address + 1) as u32;
    let byte2 = self.read_u8(address + 2) as u32;
    let byte3 = self.read_u8(address + 3) as u32;
    (byte3 << 24) | (byte2 << 16) | (byte1 << 8) | byte0
}
```

**Example**:
```
Address 0x1000: 0x78
Address 0x1001: 0x56
Address 0x1002: 0x34
Address 0x1003: 0x12

read_u32(0x1000) returns: 0x12345678
```

### Write Operations

#### Write Byte (8-bit)
```rust
fn write_u8(&mut self, address: u32, value: u8) {
    self.memory[address as usize] = value;
}
```

#### Write Word (16-bit, little-endian)
```rust
fn write_u16(&mut self, address: u32, value: u16) {
    self.write_u8(address, (value & 0xFF) as u8);        // LSB first
    self.write_u8(address + 1, ((value >> 8) & 0xFF) as u8);  // MSB second
}
```

**Example**:
```
write_u16(0x1000, 0xABCD)

Result:
Address 0x1000: 0xCD  (low byte)
Address 0x1001: 0xAB  (high byte)
```

#### Write Dword (32-bit, little-endian)
```rust
fn write_u32(&mut self, address: u32, value: u32) {
    self.write_u8(address,     (value & 0xFF) as u8);
    self.write_u8(address + 1, ((value >> 8) & 0xFF) as u8);
    self.write_u8(address + 2, ((value >> 16) & 0xFF) as u8);
    self.write_u8(address + 3, ((value >> 24) & 0xFF) as u8);
}
```

**Example**:
```
write_u32(0x2000, 0x12345678)

Result:
Address 0x2000: 0x78  (byte 0, LSB)
Address 0x2001: 0x56  (byte 1)
Address 0x2002: 0x34  (byte 2)
Address 0x2003: 0x12  (byte 3, MSB)
```

---

## Bounds Checking and Security

### Why Bounds Checking?

**Problem**: What if we try to access address `0x200000000` (8 GB) in our 4 GB emulator?

**Without bounds checking**: Rust panics or we get undefined behavior
**With bounds checking**: We return an error gracefully

### Implementation

```rust
fn read_u8(&self, address: u32) -> Result<u8, MemoryError> {
    // Check if address is in bounds
    if (address as usize) >= self.memory.len() {
        return Err(MemoryError::OutOfBounds(address));
    }
    
    // Check if address is in NULL region (security)
    if address < 0x1000 {
        return Err(MemoryError::NullPointerAccess(address));
    }
    
    // Handle MMIO specially
    if address >= 0xFFFF0000 {
        return self.read_mmio(address);
    }
    
    // Normal memory access
    Ok(self.memory[address as usize])
}
```

### Security Considerations

1. **NULL pointer detection**: Reject accesses to 0x0000-0x0FFF
2. **Bounds checking**: Reject accesses beyond our memory size
3. **Write protection**: Code section could be read-only
4. **Stack overflow detection**: Check if ESP goes too low
5. **Quota enforcement**: Limit number of memory accesses per execution

---

## Rust Implementation with Vec<u8>

### Why Vec<u8>?

```rust
pub struct Memory {
    ram: Vec<u8>,  // ← Vector of bytes
}
```

**Why not array `[u8; 4GB]`?**
- Arrays are stack-allocated → would overflow stack
- 4 GB is huge, might not fit in memory
- We might want smaller or larger memory sizes

**Why Vec?**
- ✅ Heap-allocated (can be large)
- ✅ Resizable (can adjust memory size)
- ✅ Indexable like an array (`ram[address]`)
- ✅ Efficient (contiguous memory)

### Creating Memory

```rust
impl Memory {
    pub fn new(size: usize) -> Self {
        Memory {
            ram: vec![0; size],  // Vector of 'size' zeros
        }
    }
    
    // Common sizes:
    // 64 KB:   new(64 * 1024)
    // 1 MB:    new(1024 * 1024)
    // 16 MB:   new(16 * 1024 * 1024)
    // 4 GB:    new(0x1_0000_0000)  // Might not fit in memory!
}
```

**For development**: Start with 16 MB, expand later if needed.

### Memory Access Pattern

```rust
pub struct Memory {
    ram: Vec<u8>,
    mmio: IoDevices,  // Separate struct for I/O devices
}

impl Memory {
    pub fn read_u32(&self, address: u32) -> u32 {
        // Check if MMIO
        if address >= 0xFFFF0000 {
            return self.mmio.read(address);
        }
        
        // Read from RAM (little-endian)
        let addr = address as usize;
        let byte0 = self.ram[addr] as u32;
        let byte1 = self.ram[addr + 1] as u32;
        let byte2 = self.ram[addr + 2] as u32;
        let byte3 = self.ram[addr + 3] as u32;
        
        (byte3 << 24) | (byte2 << 16) | (byte1 << 8) | byte0
    }
    
    pub fn write_u32(&mut self, address: u32, value: u32) {
        // Check if MMIO
        if address >= 0xFFFF0000 {
            self.mmio.write(address, value);
            return;
        }
        
        // Write to RAM (little-endian)
        let addr = address as usize;
        self.ram[addr]     = (value & 0xFF) as u8;
        self.ram[addr + 1] = ((value >> 8) & 0xFF) as u8;
        self.ram[addr + 2] = ((value >> 16) & 0xFF) as u8;
        self.ram[addr + 3] = ((value >> 24) & 0xFF) as u8;
    }
}
```

---

## Summary

### Key Takeaways

1. **Memory is a large array of bytes** - Each byte has a unique address (0 to 4GB in 32-bit)

2. **x86 uses little-endian** - Least significant byte first in memory

3. **Memory is organized into regions**:
   - Code (instructions)
   - Data (global variables)
   - Stack (function calls, local variables)
   - Heap (dynamic allocation)
   - MMIO (I/O devices)

4. **The stack grows downward** - PUSH decrements ESP, POP increments ESP

5. **MMIO lets us control devices** - Memory addresses 0xFFFF0000+ control LEDs, switches, etc.

6. **Multi-byte values span multiple addresses** - 32-bit value needs 4 consecutive bytes

7. **Bounds checking is critical** - Prevent crashes and security issues

8. **Vec<u8> simulates RAM** - Efficient, flexible, and easy to use in Rust

### What's Next?

Now that you understand registers, flags, and memory, you're ready to:
- **Implementation guides**: How to write the actual Rust code
- **Instructions**: How operations like ADD manipulate these components
- **Decoder**: How to parse instruction bytes from memory
- **Executor**: How to run the fetch-decode-execute cycle

You now have the foundational knowledge needed to build the emulator!
