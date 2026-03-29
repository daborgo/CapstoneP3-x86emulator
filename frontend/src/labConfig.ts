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
    title: 'Lab 2: Loops (Conditional and Unconditional Branch Instructions)',
    description:
      'Write an x86 program to remove all occurrences of a given element from an array.\n\n' +
      'Memory layout:\n' +
      '  [0x1F00] = n    (array length, update this after removal)\n' +
      '  [0x1F04] = val  (value to remove)\n' +
      '  Array A[] starts at address 0x2000 (each element is 4 bytes)\n\n' +
      'C pseudocode:\n' +
      '  int i = 0;\n' +
      '  while (i < n) {\n' +
      '    if (A[i] != val)\n' +
      '      i++;\n' +
      '    else {\n' +
      '      for (int k = i; k < n - 1; k++)\n' +
      '        A[k] = A[k+1];\n' +
      '      n = n - 1;\n' +
      '    }\n' +
      '  }\n\n' +
      'After execution, store the new n back to [0x1F00].\n' +
      'Only the first n elements of the array will be checked.\n\n' +
      'Grading: 4 automated tests (5 pts each) + 2 manual checks (5 pts each) = 30 pts.',
    starterCode: `; Lab 2: Loops - Remove element from array
; Memory layout:
;   [0x1F00] = n   (array length)
;   [0x1F04] = val (value to remove)
;   Array A[] at 0x2000: A[0]=[0x2000], A[1]=[0x2004], ...
;
; TODO: Implement the removal loop
;   1. Load n from [0x1F00], val from [0x1F04]
;   2. Loop through array, remove all occurrences of val
;      by shifting subsequent elements left
;   3. Store updated n back to [0x1F00]

; Load parameters
MOV ECX, [0x1F00]         ; ECX = n (array length)
MOV EDX, [0x1F04]         ; EDX = val (value to remove)
MOV EBX, 0x00002000       ; EBX = base address of A[]

; TODO: implement your removal loop here
;   Use CMP, JE/JNE, JL/JGE for comparisons and branches
;   Use JMP for unconditional jumps

; When done, store the new n
MOV [0x1F00], ECX         ; store updated n
`,
  },
  3: {
    id: 3,
    title: 'Lab 3: Single Procedure Call',
    description:
      'Given an array A of at least one integer, create a new array B where B[i] = A[i]^i.\n\n' +
      'Write two functions called from main using CALL:\n' +
      '  - exponent(x, y): returns x raised to the power y (A[i]^i)\n' +
      '  - append(B, n2, exp): stores exp at B[n2]\n\n' +
      'Memory layout:\n' +
      '  [0x1F00] = n1   (length of array A)\n' +
      '  Array A[] starts at address 0x2000 (each element is 4 bytes)\n' +
      '  Array B[] starts at address 0x3000 (output, each element is 4 bytes)\n' +
      '  Store final n2 (length of B) at [0x1F04]\n\n' +
      'Algorithm (C pseudocode):\n' +
      '  B[0] = 1;  // A[0]^0 = 1\n' +
      '  for (j = 1; j < n1; j++) {\n' +
      '    n2 = j;\n' +
      '    exp = exponent(A[j], j);\n' +
      '    append(B, n2, exp);\n' +
      '  }\n' +
      '  n2++;\n\n' +
      'Grading: 4 automated tests (5 pts each) + 2 manual checks (5 pts each) = 30 pts.',
    starterCode: `; Lab 3: Single Procedure Call
; Memory layout:
;   [0x1F00] = n1  (array length)
;   Array A[] at 0x2000: A[0]=[0x2000], A[1]=[0x2004], ...
;   Array B[] at 0x3000: B[0]=[0x3000], B[1]=[0x3004], ...
;   Store result n2 at [0x1F04]
;
; Instructions available: CALL, RET, PUSH, POP, IMUL
; Use CALL label / RET for procedure calls

; ── main ────────────────────────────────────
MOV ECX, [0x1F00]         ; ECX = n1 (array length)
MOV EBX, 0x00002000       ; EBX = base address of A[]
MOV EDI, 0x00003000       ; EDI = base address of B[]

; B[0] = 1 (A[0]^0 is always 1)
MOV EAX, 1
MOV [EDI], EAX

; TODO: implement the main loop
;   for j = 1 to n1-1:
;     compute exp = exponent(A[j], j)
;     call append(B, j, exp)
;   store final n2 at [0x1F04]

; Store n2 (should equal n1 when done)
MOV [0x1F04], ECX

; TODO: implement exponent and append functions
; exponent: takes x (in EAX) and y (in ECX), returns result in EAX
; append: takes B base (in EDI), index n2 (in ESI), value (in EAX)
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
