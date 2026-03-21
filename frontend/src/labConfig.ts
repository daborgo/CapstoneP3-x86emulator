export interface LabConfig {
  id: number
  title: string
  description: string
  starterCode: string
}

export const LAB_COUNT = 5

export const labConfigs: Record<number, LabConfig> = {
  1: {
    id: 1,
    title: 'Lab 1: ALU and Data Transfer Instructions',
    description:
      'Array A[] is stored in memory starting at address 0x2000 (each element is 4 bytes).\n' +
      'A[0] and A[1] are given. Compute:\n' +
      '  {A[3], A[2]} = A[1] * A[0]  (unsigned 64-bit multiply)\n' +
      '  A[4] = A[2] / 230            (signed divide)\n' +
      '  a    = A[2] % 230            (signed remainder)\n' +
      '  b = (char)(a >> 16),  c = (char)(a & 8),  d = (short)(a << 2)\n' +
      '  A[5] = {b, c, d}             (b=bits 31-24, c=bits 23-16, d=bits 15-0)\n\n' +
      'Submit to test all 3 graded cases.',
    starterCode: `; Lab 1: ALU and Data Transfer Instructions
; Array A[] is at base address 0x2000
;   A[0] = [0x2000], A[1] = [0x2004], A[2] = [0x2008]
;   A[3] = [0x200C], A[4] = [0x2010], A[5] = [0x2014]

MOV EBX, 0x00002000       ; EBX = base address of A[]

; Step 1: unsigned multiply {A[3], A[2]} = A[1] * A[0]
MOV EAX, [EBX]            ; EAX = A[0]
MOV ECX, [EBX+4]          ; ECX = A[1]
MUL ECX                   ; EDX:EAX = EAX * ECX (unsigned)
MOV [EBX+8],  EAX         ; A[2] = low 32 bits
MOV [EBX+12], EDX         ; A[3] = high 32 bits

; Step 2: A[4] = A[2] / 230,  a (EDX) = A[2] % 230
MOV EAX, [EBX+8]          ; EAX = A[2]
CDQ                        ; sign-extend EAX into EDX:EAX
MOV ECX, 230
IDIV ECX                  ; EAX = quotient, EDX = remainder (a)
MOV [EBX+16], EAX         ; A[4]

; Step 3: b = (char)(a >> 16), c = (char)(a & 8), d = (short)(a << 2)
; EDX holds 'a' here
MOV EAX, EDX              ; EAX = a (for b)
SAR EAX, 16               ; arithmetic shift right 16
AND EAX, 0x000000FF       ; b = low byte (char truncation)
MOV ECX, EDX              ; ECX = a (for c)
AND ECX, 8                ; c = a & 8
MOV ESI, EDX              ; ESI = a (for d)
SHL ESI, 2                ; d = a << 2
AND ESI, 0x0000FFFF       ; d = short truncation

; Step 4: A[5] = {b, c, d} packed into 32 bits
SHL EAX, 24               ; b in bits 31-24
SHL ECX, 16               ; c in bits 23-16
OR  EAX, ECX
OR  EAX, ESI
MOV [EBX+20], EAX         ; A[5]
`,
  },
  2: {
    id: 2,
    title: 'Lab 2: Arithmetic (ADD & SUB)',
    description:
      'Perform basic arithmetic operations using ADD and SUB instructions. ' +
      'Observe how each operation affects the CPU flags (ZF, SF, CF, OF).',
    starterCode: `; Lab 2: Arithmetic
; Use ADD and SUB instructions as directed.

MOV EAX, 0x0000000A
MOV EBX, 0x00000005
ADD EAX, EBX
SUB EAX, 0x00000003
`,
  },
  3: {
    id: 3,
    title: 'Lab 3: Stack (PUSH & POP)',
    description:
      'Explore the x86 stack using PUSH and POP instructions. ' +
      'Observe how ESP changes with each push and pop operation.',
    starterCode: `; Lab 3: Stack Operations
; Use PUSH and POP to manipulate the stack.

MOV EAX, 0x000000FF
PUSH EAX
MOV EAX, 0x00000000
POP EAX
`,
  },
  4: {
    id: 4,
    title: 'Lab 4: Control Flow (JMP)',
    description:
      'Implement unconditional control flow using JMP instructions ' +
      'to branch between sections of code.',
    starterCode: `; Lab 4: Control Flow
; Use JMP to control the execution path.

MOV EAX, 0x00000001
MOV EBX, 0x00000002
`,
  },
  5: {
    id: 5,
    title: 'Lab 5: Functions (CALL & RET)',
    description:
      'Implement subroutines using CALL and RET instructions, ' +
      'managing the call stack properly.',
    starterCode: `; Lab 5: Functions
; Implement the required subroutines using CALL and RET.

MOV EAX, 0x00000001
MOV EBX, 0x00000002
`,
  },
}
