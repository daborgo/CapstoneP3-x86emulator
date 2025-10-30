# Understanding CPU Flags

## Table of Contents
1. [What Are Flags?](#what-are-flags)
2. [The EFLAGS Register](#the-eflags-register)
3. [The Six Key Flags](#the-six-key-flags)
4. [Flag Calculations in Detail](#flag-calculations-in-detail)
5. [How Flags Enable Decisions](#how-flags-enable-decisions)
6. [Binary Examples](#binary-examples)
7. [Flag Interactions](#flag-interactions)
8. [Rust Implementation Choices](#rust-implementation-choices)

---

## What Are Flags?

### Simple Definition
**Flags are single bits that indicate the result or status of CPU operations.** They answer questions like "Was the result zero?" or "Did we overflow?"

### The Analogy
Imagine you're a teacher grading a test:
- **The test score** = The actual result (like the value in EAX)
- **Check marks** = The flags:
  - ✓ "Student passed" (not zero)
  - ✓ "Perfect score" (no overflow)
  - ✓ "Needs improvement" (negative result)

Just like check marks give you quick information about the test without re-reading it, flags give the CPU quick information about the last operation without recalculating.

### Key Characteristics
- **Status Indicators**: They tell you ABOUT a result, not the result itself
- **Side Effects**: Most instructions automatically update flags
- **Decision Making**: Used by conditional jumps (JE, JNE, JG, etc.)
- **Single Bits**: Each flag is just 1 bit (true/false, set/clear, 1/0)
- **Stored in EFLAGS**: All flags live in one 32-bit register

---

## The EFLAGS Register

### What is EFLAGS?
EFLAGS (Extended FLAGS) is a special 32-bit register where each bit has a specific meaning. It's not used for calculations—it stores status information.

### EFLAGS Bit Layout (32 bits)
```
31                                                              0
┌──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┐
│  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │OF│DF│IF│TF│SF│ZF│  │AF│  │PF│  │CF│
└──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘
                                                    11 10 9  8  7  6     4     2     0
```

### The Flags We Care About
For arithmetic and logic operations, these six flags are critical:

| Bit | Flag | Name | Purpose |
|-----|------|------|---------|
| 0 | **CF** | Carry Flag | Unsigned overflow (carry out of MSB) |
| 2 | **PF** | Parity Flag | Even number of 1-bits in result |
| 4 | **AF** | Auxiliary Carry | Carry from bit 3 to bit 4 (BCD arithmetic) |
| 6 | **ZF** | Zero Flag | Result is zero |
| 7 | **SF** | Sign Flag | Result is negative (bit 31 = 1) |
| 11 | **OF** | Overflow Flag | Signed overflow occurred |

**Note**: There are other flags (DF for direction, IF for interrupts, TF for trap) but we'll focus on arithmetic flags first.

---

## The Six Key Flags

### CF - Carry Flag (Bit 0)

**Question it answers**: "Did unsigned arithmetic overflow?"

**When it's set (CF=1)**:
- **Addition**: The result was too big to fit in 32 bits
- **Subtraction**: Had to "borrow" (result would be negative in unsigned)

**Example (Addition)**:
```
  0xFFFFFFFF  (4,294,967,295 - largest 32-bit unsigned)
+ 0x00000001  (1)
= 0x00000000  (wraps around!)
  
CF = 1  ← Set because we carried out of bit 31
```

**Example (Subtraction)**:
```
  0x00000005  (5)
- 0x0000000A  (10)
= 0xFFFFFFFB  (would be -5 in signed, but we're thinking unsigned)

CF = 1  ← Set because we had to "borrow"
```

**Use case**: Multi-precision arithmetic (numbers larger than 32 bits)

### ZF - Zero Flag (Bit 6)

**Question it answers**: "Was the result exactly zero?"

**When it's set (ZF=1)**:
- The result of the operation equals zero

**Example**:
```
  0x00000005  (5)
- 0x00000005  (5)
= 0x00000000  (0)

ZF = 1  ← Set because result is zero
```

**Use case**: 
- Testing equality: `CMP EAX, EBX` followed by `JE` (jump if equal)
- Loop termination: `DEC ECX` followed by `JZ` (jump if zero)

**Most commonly used flag!**

### SF - Sign Flag (Bit 7)

**Question it answers**: "Is the result negative (in two's complement)?"

**When it's set (SF=1)**:
- Bit 31 of the result is 1

**Example**:
```
  0x00000005  (5)
- 0x0000000A  (10)
= 0xFFFFFFFB  (-5 in two's complement)

SF = 1  ← Set because bit 31 is 1
```

**How it works**: It literally just copies bit 31 of the result!
```
SF = (result >> 31) & 1
```

**Use case**: Signed comparisons (`JL`, `JG`, etc.)

### OF - Overflow Flag (Bit 11)

**Question it answers**: "Did signed arithmetic overflow?"

**When it's set (OF=1)**:
- **Addition**: Positive + Positive = Negative, OR Negative + Negative = Positive
- **Subtraction**: Positive - Negative = Negative, OR Negative - Positive = Positive

**Example (Positive + Positive = Negative)**:
```
  0x7FFFFFFF  (2,147,483,647 - largest positive 32-bit signed)
+ 0x00000001  (1)
= 0x80000000  (-2,147,483,648 - becomes negative!)

OF = 1  ← Set because positive + positive gave negative (overflow!)
SF = 1  ← Set because result is negative
CF = 0  ← Clear because no unsigned overflow
```

**Example (Negative + Negative = Positive)**:
```
  0x80000000  (-2,147,483,648 - smallest negative 32-bit signed)
+ 0x80000000  (-2,147,483,648)
= 0x00000000  (0 - should be -4,294,967,296!)

OF = 1  ← Set because negative + negative gave positive (overflow!)
```

**Use case**: Detecting errors in signed arithmetic

### PF - Parity Flag (Bit 2)

**Question it answers**: "Does the lowest byte have an even number of 1-bits?"

**When it's set (PF=1)**:
- The lowest 8 bits of the result contain an even number of 1-bits

**Example**:
```
Result = 0x12345678
Lowest byte = 0x78 = 01111000 in binary
                     ↑↑↑↑     = 4 one-bits (even)
PF = 1  ← Set because even number of 1-bits
```

**Example**:
```
Result = 0x1234567F
Lowest byte = 0x7F = 01111111 in binary
                     ↑↑↑↑↑↑↑  = 7 one-bits (odd)
PF = 0  ← Clear because odd number of 1-bits
```

**Use case**: Error detection in communication protocols (rarely used in modern code)

### AF - Auxiliary Carry Flag (Bit 4)

**Question it answers**: "Did we carry from bit 3 to bit 4?"

**When it's set (AF=1)**:
- There was a carry from the lower nibble (bits 0-3) to the upper nibble (bits 4-7)

**Example**:
```
  0x0000000F  (bits 0-3 are all 1s)
+ 0x00000001
= 0x00000010  (carried into bit 4)

AF = 1  ← Set because we carried from bit 3 to bit 4
```

**Use case**: BCD (Binary Coded Decimal) arithmetic with DAA/DAS instructions (rarely used)

---

## Flag Calculations in Detail

### CF (Carry Flag) Calculation

#### For Addition (a + b = result)
```rust
let result = a.wrapping_add(b);
let cf = result < a;  // If result is less than operand, we wrapped around
```

**Why this works**: If `result < a`, that means we added something to `a` but got a smaller number, which only happens on overflow.

**Example**:
```
a = 0xFFFFFFF0  (4,294,967,280)
b = 0x00000020  (32)
result = 0x10   (16) - wrapped around!

result < a? Yes! (16 < 4,294,967,280)
Therefore: CF = 1
```

#### For Subtraction (a - b = result)
```rust
let result = a.wrapping_sub(b);
let cf = a < b;  // If a is less than b, we had to "borrow"
```

**Example**:
```
a = 0x00000005  (5)
b = 0x0000000A  (10)
result = 0xFFFFFFFB  (-5 in two's complement, or 4,294,967,291 unsigned)

a < b? Yes! (5 < 10)
Therefore: CF = 1
```

### ZF (Zero Flag) Calculation

**Simplest flag**:
```rust
let zf = (result == 0);
```

That's it! If result is zero, ZF = true/1. Otherwise ZF = false/0.

### SF (Sign Flag) Calculation

**Check if bit 31 is set**:
```rust
let sf = (result & 0x8000_0000) != 0;
// Or equivalently:
let sf = (result as i32) < 0;
```

**Binary breakdown**:
```
0x8000_0000 = 10000000000000000000000000000000 (only bit 31 set)

If result =   01111111111111111111111111111111 (0x7FFFFFFF)
result & mask = 00000000000000000000000000000000 (0) → SF = 0

If result =   10000000000000000000000000000000 (0x80000000)
result & mask = 10000000000000000000000000000000 (not 0) → SF = 1
```

### OF (Overflow Flag) Calculation

**Most complex flag!** For addition:

```rust
// Check if signs of operands match AND sign of result differs
let operand_signs_match = (a ^ b) & 0x8000_0000 == 0;
let result_sign_differs = (a ^ result) & 0x8000_0000 != 0;
let of = operand_signs_match && result_sign_differs;
```

**What this means**:
1. **Operand signs match**: Both positive OR both negative
2. **Result sign differs**: Result has different sign than operands
3. **Overflow if both true**: Pos + Pos = Neg, OR Neg + Neg = Pos

**Example walkthrough**:
```
a = 0x7FFFFFFF  (positive, bit 31 = 0)
b = 0x00000001  (positive, bit 31 = 0)
result = 0x80000000  (negative, bit 31 = 1)

Step 1: Do operand signs match?
a =        01111111111111111111111111111111
b =        00000000000000000000000000000001
a XOR b =  01111111111111111111111111111110
           ↑
           Bit 31 = 0, so signs match (both positive)

Step 2: Does result sign differ from operands?
a =        01111111111111111111111111111111 (bit 31 = 0)
result =   10000000000000000000000000000000 (bit 31 = 1)
a XOR res= 11111111111111111111111111111111
           ↑
           Bit 31 = 1, so result sign differs

Step 3: Both conditions true?
Signs match? YES
Result differs? YES
Therefore: OF = 1 (overflow occurred!)
```

### PF (Parity Flag) Calculation

**Count 1-bits in lowest byte**:
```rust
let lowest_byte = (result & 0xFF) as u8;
let one_bits = lowest_byte.count_ones();
let pf = (one_bits % 2) == 0;  // Even number of 1-bits
```

**Example**:
```
result = 0x12345678
lowest_byte = 0x78 = 01111000

Count 1-bits: 0+1+1+1+1+0+0+0 = 4 (even)
Therefore: PF = 1
```

### AF (Auxiliary Carry Flag) Calculation

**Check carry from bit 3 to bit 4**:
```rust
// For addition:
let af = ((a ^ b ^ result) & 0x10) != 0;
```

**What this does**: The XOR operations reveal which bits changed, and we check if bit 4 changed (which indicates a carry from bit 3).

---

## How Flags Enable Decisions

### Conditional Jumps
Flags are the key to making decisions in assembly language:

```assembly
CMP EAX, EBX        ; Compare EAX and EBX (internally: EAX - EBX)
                    ; This sets flags but doesn't store result
JE  equal_label     ; Jump if Equal (if ZF=1)
JNE not_equal       ; Jump if Not Equal (if ZF=0)
JG  greater         ; Jump if Greater (signed: ZF=0 AND SF=OF)
JL  less            ; Jump if Less (signed: SF!=OF)
```

### Example: IF Statement

**C code**:
```c
if (a == b) {
    x = 1;
} else {
    x = 2;
}
```

**Assembly**:
```assembly
    MOV EAX, [a]        ; Load a
    MOV EBX, [b]        ; Load b
    CMP EAX, EBX        ; Compare (sets ZF if equal)
    JNE else_block      ; Jump to else if not equal (ZF=0)
    
    ; Then block
    MOV [x], 1          ; x = 1
    JMP end_if
    
else_block:
    MOV [x], 2          ; x = 2
    
end_if:
    ; Continue...
```

### Example: WHILE Loop

**C code**:
```c
while (counter > 0) {
    counter--;
}
```

**Assembly**:
```assembly
loop_start:
    CMP ECX, 0          ; Compare counter to 0
    JLE loop_end        ; Jump if Less or Equal (ZF=1 OR SF!=OF)
    
    DEC ECX             ; counter--
    JMP loop_start
    
loop_end:
    ; Continue...
```

### Conditional Moves
Modern x86 has conditional move instructions that use flags:

```assembly
MOV EAX, 10
MOV EBX, 20
CMP EAX, EBX
CMOVG EAX, EBX      ; If EAX > EBX, move EBX to EAX (conditionally)
                     ; Uses flags set by CMP
```

---

## Binary Examples

### Example 1: Simple Addition
```
Operation: 5 + 3 = 8

Binary:
  00000000000000000000000000000101  (5)
+ 00000000000000000000000000000011  (3)
= 00000000000000000000000000001000  (8)

Flags:
CF = 0  (no carry out of bit 31)
ZF = 0  (result is not zero)
SF = 0  (bit 31 is 0, so positive)
OF = 0  (no signed overflow: pos + pos = pos ✓)
PF = 0  (lowest byte 0x08 = 00001000 = 1 one-bit, which is odd)
AF = 0  (no carry from bit 3 to bit 4)
```

### Example 2: Zero Result
```
Operation: 10 - 10 = 0

Binary:
  00000000000000000000000000001010  (10)
- 00000000000000000000000000001010  (10)
= 00000000000000000000000000000000  (0)

Flags:
CF = 0  (10 >= 10, no borrow)
ZF = 1  ← Result is zero!
SF = 0  (bit 31 is 0)
OF = 0  (no overflow: pos - pos = zero ✓)
PF = 1  (0x00 = 00000000 = 0 one-bits, which is even)
AF = 0  (no borrow from bit 4 to bit 3)
```

### Example 3: Unsigned Overflow (Carry)
```
Operation: 0xFFFFFFFF + 1 = 0 (with carry)

Binary:
  11111111111111111111111111111111  (4,294,967,295)
+ 00000000000000000000000000000001  (1)
= 00000000000000000000000000000000  (0, wrapped around!)
  ↑
  Carried out!

Flags:
CF = 1  ← Unsigned overflow!
ZF = 1  (result is zero)
SF = 0  (bit 31 is 0)
OF = 0  (no signed overflow: neg + pos = zero ✓)
PF = 1  (0x00 = even number of 1-bits)
AF = 1  (carried from bit 3 to bit 4)
```

### Example 4: Signed Overflow
```
Operation: 0x7FFFFFFF + 1 = 0x80000000

Binary:
  01111111111111111111111111111111  (2,147,483,647 - max positive)
+ 00000000000000000000000000000001  (1)
= 10000000000000000000000000000000  (-2,147,483,648)
  ↑
  Flipped to negative!

Flags:
CF = 0  (no unsigned overflow - result is valid unsigned)
ZF = 0  (result is not zero)
SF = 1  (bit 31 is 1, negative)
OF = 1  ← Signed overflow! (pos + pos = neg ✗)
PF = 1  (0x00 in lowest byte = even)
AF = 0  (no carry into bit 4)
```

### Example 5: Negative Result
```
Operation: 5 - 10 = -5

Binary (two's complement subtraction):
  00000000000000000000000000000101  (5)
- 00000000000000000000000000001010  (10)
= 11111111111111111111111111111011  (-5 in two's complement)

Flags:
CF = 1  (5 < 10, so borrow occurred in unsigned)
ZF = 0  (result is not zero)
SF = 1  ← Result is negative (bit 31 = 1)
OF = 0  (no signed overflow: pos - pos = neg ✓)
PF = 1  (0xFB = 11111011 = 6 one-bits, which is even)
AF = 1  (borrow from bit 4)
```

---

## Flag Interactions

### CF vs OF: What's the Difference?

**CF (Carry Flag)**: For **unsigned** arithmetic
**OF (Overflow Flag)**: For **signed** arithmetic

**Same operation, different interpretations**:
```
  0xFFFFFFFF + 0x00000002 = 0x00000001

As UNSIGNED:
  4,294,967,295 + 2 = 4,294,967,297 (doesn't fit in 32 bits!)
  CF = 1  ← Unsigned overflow!
  OF = 0  ← No signed overflow

As SIGNED (two's complement):
  -1 + 2 = 1 (perfectly valid!)
  OF = 0  ← No signed overflow
  CF = 1  ← Unsigned would overflow
```

**Use CF when**: Working with unsigned numbers (addresses, sizes, etc.)
**Use OF when**: Working with signed numbers (temperatures, balances, etc.)

### ZF and SF Together

Common combinations:
```
ZF=1, SF=0: Result is exactly zero (0x00000000)
ZF=0, SF=0: Result is positive (0x00000001 to 0x7FFFFFFF)
ZF=0, SF=1: Result is negative (0x80000000 to 0xFFFFFFFF)
ZF=1, SF=1: Never happens! (Zero has bit 31 = 0)
```

### Multiple Flags for Comparisons

**Signed comparisons** use multiple flags:
```
JG  (Jump if Greater):           ZF=0 AND SF=OF
JGE (Jump if Greater or Equal):  SF=OF
JL  (Jump if Less):              SF!=OF
JLE (Jump if Less or Equal):     ZF=1 OR SF!=OF
```

**Why SF=OF for equality in signed?**
- If OF=0, then SF correctly indicates sign of result
- If OF=1, then SF is inverted from what it should be
- SF=OF means "taking overflow into account, result is positive/zero"

---

## Rust Implementation Choices

### bool vs Bitfield

**Option 1: Individual booleans** (We'll use this)
```rust
pub struct Flags {
    pub cf: bool,  // Carry
    pub zf: bool,  // Zero
    pub sf: bool,  // Sign
    pub of: bool,  // Overflow
    pub pf: bool,  // Parity
    pub af: bool,  // Auxiliary Carry
}
```

**Advantages**:
- ✅ Easy to read and understand
- ✅ Easy to modify individual flags
- ✅ Rust's bool is well-optimized
- ✅ Clear intent in code

**Disadvantages**:
- ❌ Uses more memory (6 bytes vs 4 bytes for u32)
- ❌ Doesn't match hardware representation exactly

**Option 2: Bitfield in u32** (More hardware-accurate)
```rust
pub struct Flags {
    bits: u32,  // Bit 0 = CF, Bit 2 = PF, etc.
}

impl Flags {
    pub fn cf(&self) -> bool { (self.bits & 0x0001) != 0 }
    pub fn set_cf(&mut self, value: bool) {
        if value {
            self.bits |= 0x0001;   // Set bit 0
        } else {
            self.bits &= !0x0001;  // Clear bit 0
        }
    }
}
```

**Advantages**:
- ✅ Matches hardware exactly
- ✅ Uses less memory
- ✅ Can save/restore entire EFLAGS at once

**Disadvantages**:
- ❌ More complex code
- ❌ Bitwise operations are harder to read
- ❌ More error-prone

**Our choice**: Start with booleans for clarity. Optimize later if needed.

### Why Not use bitflags crate?

Rust has a popular `bitflags` crate, but for learning, we'll implement it ourselves to understand the concepts fully.

---

## Summary

### Key Takeaways

1. **Flags are status indicators** - They tell you ABOUT the result, not the result itself

2. **Six main flags for arithmetic**:
   - **CF**: Unsigned overflow
   - **ZF**: Result is zero
   - **SF**: Result is negative
   - **OF**: Signed overflow
   - **PF**: Even parity (rarely used)
   - **AF**: Auxiliary carry (rarely used)

3. **Flags enable decisions** - Conditional jumps use flags to implement IF statements and loops

4. **Different flags for different interpretations**:
   - CF for unsigned arithmetic
   - OF for signed arithmetic
   - Both can be set for the same operation!

5. **Most instructions update flags** - It's a "side effect" of arithmetic/logic operations

6. **Flags are stored in EFLAGS register** - But we'll implement as separate booleans for clarity

### What's Next?

Now that you understand flags, you can move on to:
- **Memory**: How the CPU accesses RAM and I/O devices
- **Instructions**: How operations like ADD actually modify flags
- **Implementation**: Writing the actual Rust code for the Flags struct

Understanding flags is crucial because almost every instruction modifies them!
