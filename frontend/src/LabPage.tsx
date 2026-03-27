import { useState, useEffect, useRef } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { labConfigs } from './labConfig'
import GradingPanel, { type GradingResult } from './GradingPanel'
import { saveStudentSubmission } from './submissionsStore'
import './App.css'

type WasmModule = typeof import('./wasm/pkg/web_x86_core')
type EmulatorApi = import('./wasm/pkg/web_x86_core').Emulator

const LOAD_ADDR = 0x00001000

// ─── Register index helpers ───────────────────────────────────────────────────

function regIndex(r: string): number {
  switch (r.toUpperCase()) {
    case 'EAX': return 0
    case 'ECX': return 1
    case 'EDX': return 2
    case 'EBX': return 3
    case 'ESP': return 4
    case 'EBP': return 5
    case 'ESI': return 6
    case 'EDI': return 7
    default: return -1
  }
}

function toNum(tok: string): number | null {
  const t = tok.trim()
  if (/^0x[0-9a-f]+$/i.test(t)) return parseInt(t, 16) >>> 0
  if (/^-?[0-9]+$/.test(t)) return parseInt(t, 10) >>> 0
  return null
}

// ─── Memory operand parser ────────────────────────────────────────────────────

interface MemRef {
  type: 'abs'              // [0x2000]
  addr: number
}
interface MemReg {
  type: 'reg'              // [EBX]
  reg: number
}
interface MemRegDisp {
  type: 'reg_disp'         // [EBX+4] or [EBX-4]
  reg: number
  disp: number             // signed
}
type MemOperand = MemRef | MemReg | MemRegDisp

function parseMemOperand(tok: string): MemOperand | null {
  const m = tok.match(/^\[(.+)\]$/)
  if (!m) return null
  const inner = m[1].trim()

  // Absolute address: [0x2000] or [8192]
  const numMatch = inner.match(/^(0x[0-9a-f]+|[0-9]+)$/i)
  if (numMatch) {
    const addr = parseInt(numMatch[1], numMatch[1].toLowerCase().startsWith('0x') ? 16 : 10) >>> 0
    return { type: 'abs', addr }
  }

  // Pure register: [EBX]
  const rIdx = regIndex(inner)
  if (rIdx >= 0) return { type: 'reg', reg: rIdx }

  // Register ± offset: [EBX+4], [EBX+0x10], [EBX-8]
  const rdm = inner.match(/^(\w+)\s*([+-])\s*(.+)$/)
  if (rdm) {
    const rIdx2 = regIndex(rdm[1])
    if (rIdx2 >= 0) {
      const sign = rdm[2] === '-' ? -1 : 1
      const dispStr = rdm[3].trim()
      const dispAbs = parseInt(dispStr, dispStr.toLowerCase().startsWith('0x') ? 16 : 10)
      if (!isNaN(dispAbs)) return { type: 'reg_disp', reg: rIdx2, disp: sign * dispAbs }
    }
  }
  return null
}

// ─── Encode memory operand bytes (ModRM + optional disp) ─────────────────────
// Returns the bytes to append after the opcode byte when the rm is this mem operand.
// regField: the 3-bit field for the "other" register (src or dst) that goes in reg bits.

function encodeRmMem(mem: MemOperand, regField: number): number[] {
  if (mem.type === 'abs') {
    // mod=00, rm=5 (disp32)
    const a = mem.addr >>> 0
    const modrm = (0b00 << 6) | ((regField & 7) << 3) | 5
    return [modrm, a & 0xFF, (a >>> 8) & 0xFF, (a >>> 16) & 0xFF, (a >>> 24) & 0xFF]
  }
  if (mem.type === 'reg') {
    const rm = mem.reg
    if (rm === 4) return []  // ESP needs SIB – caller must handle
    if (rm === 5) {
      // EBP in mod=00 is disp32 only; use mod=01 disp=0 instead
      const modrm = (0b01 << 6) | ((regField & 7) << 3) | 5
      return [modrm, 0x00]
    }
    const modrm = (0b00 << 6) | ((regField & 7) << 3) | rm
    return [modrm]
  }
  // reg_disp
  const rm = mem.reg
  if (rm === 4) return []  // ESP SIB not supported
  const disp = mem.disp
  if (disp >= -128 && disp <= 127) {
    const modrm = (0b01 << 6) | ((regField & 7) << 3) | rm
    return [modrm, disp & 0xFF]
  }
  const modrm = (0b10 << 6) | ((regField & 7) << 3) | rm
  const d = disp >>> 0
  return [modrm, d & 0xFF, (d >>> 8) & 0xFF, (d >>> 16) & 0xFF, (d >>> 24) & 0xFF]
}

// ─── Assembler ─────────────────────────────────────────────────────────────────

// Conditional jump opcode map
const JCC_OPCODES: Record<string, number> = {
  JE: 0x74, JZ: 0x74, JNE: 0x75, JNZ: 0x75,
  JL: 0x7C, JNGE: 0x7C, JGE: 0x7D, JNL: 0x7D,
  JLE: 0x7E, JNG: 0x7E, JG: 0x7F, JNLE: 0x7F,
}

// Tokenize a line: strip comments, split tokens preserving [..] brackets
function tokenizeLine(raw: string): { tokens: string[]; label: string | null } {
  const line = raw.split(';')[0].trim()
  if (!line) return { tokens: [], label: null }
  if (/^(section|db|dw|dd)\b/i.test(line)) return { tokens: [], label: null }

  // Check for label (word followed by colon)
  let rest = line
  let label: string | null = null
  const labelMatch = line.match(/^(\w+)\s*:\s*(.*)$/)
  if (labelMatch) {
    label = labelMatch[1].toUpperCase()
    rest = labelMatch[2].trim()
  }
  if (!rest) return { tokens: [], label }

  const tokens: string[] = []
  let cur = ''
  let inBracket = false
  for (const ch of rest.replace(/\s*,\s*/g, ',').replace(/\s+/g, ' ')) {
    if (ch === '[') { inBracket = true; cur += ch }
    else if (ch === ']') { inBracket = false; cur += ch }
    else if ((ch === ' ' || ch === ',') && !inBracket) {
      if (cur) { tokens.push(cur); cur = '' }
    } else { cur += ch }
  }
  if (cur) tokens.push(cur)
  return { tokens, label }
}

// Calculate the byte size of an instruction (without emitting bytes)
function instrSize(op: string, tokens: string[]): number {
  if (op === 'MOV') {
    const dst = tokens[1], src2 = tokens[2]
    const dstMem = parseMemOperand(dst)
    const srcMem = parseMemOperand(src2)
    if (!dstMem && !srcMem) {
      if (regIndex(src2) >= 0) return 2                // MOV reg, reg
      return 5                                          // MOV reg, imm32
    }
    if (srcMem || dstMem) {
      const mem = srcMem || dstMem
      if (!mem) return 2
      if (mem.type === 'abs') return 6                  // opcode + modrm + disp32
      if (mem.type === 'reg') return mem.reg === 5 ? 3 : 2
      // reg_disp
      const disp = (mem as MemRegDisp).disp
      return (disp >= -128 && disp <= 127) ? 3 : 6
    }
    return 2
  }
  if (op === 'PUSH' || op === 'POP') return 1
  if (op === 'ADD' || op === 'SUB' || op === 'CMP') {
    if (regIndex(tokens[2]) >= 0) return 2              // reg, reg
    return 6                                            // reg, imm32 (0x81 form)
  }
  if (op === 'AND' || op === 'OR') {
    if (regIndex(tokens[2]) >= 0) return 2
    return 6
  }
  if (op === 'SHL' || op === 'SAL' || op === 'SHR' || op === 'SAR') return 3
  if (op === 'MUL' || op === 'IDIV') return 2
  if (op === 'IMUL') return 3                            // 0x0F 0xAF + ModRM
  if (op === 'CDQ' || op === 'RET') return 1
  if (op === 'JMP') return 2                            // always use short form for labels; will expand if needed
  if (op in JCC_OPCODES) return 2                       // conditional jumps are always rel8
  if (op === 'CALL') return 5
  return 0
}

export function assemble(src: string): { bytes: Uint8Array; errors: string[] } {
  const errors: string[] = []
  const lines = src.split('\n')

  // ── Pass 1: collect labels and instruction offsets ──────────────────────────
  interface ParsedLine { lineNum: number; tokens: string[]; label: string | null; op: string }
  const parsed: ParsedLine[] = []
  const labels = new Map<string, number>()   // label name → byte offset
  let offset = 0

  for (let i = 0; i < lines.length; i++) {
    const lineNum = i + 1
    const { tokens, label } = tokenizeLine(lines[i])

    if (label) {
      if (labels.has(label)) {
        errors.push(`Line ${lineNum}: Duplicate label '${label}'`)
      } else {
        labels.set(label, offset)
      }
    }

    if (!tokens.length) continue
    const op = tokens[0].toUpperCase()
    const size = instrSize(op, tokens)
    if (size === 0 && op !== 'CDQ' && op !== 'RET') {
      // Unknown mnemonic – will be caught in pass 2
    }
    parsed.push({ lineNum, tokens, label: null, op })
    offset += size || 1 // fallback 1 to keep offsets moving for unknown ops
  }

  // ── Pass 2: emit bytes ─────────────────────────────────────────────────────
  const out: number[] = []

  for (const { lineNum, tokens, op } of parsed) {
    const currentOffset = out.length

    // ── MOV ──────────────────────────────────────────────────────────────────
    if (op === 'MOV') {
      if (tokens.length !== 3) { errors.push(`Line ${lineNum}: MOV expects 2 operands`); continue }
      const dst = tokens[1], src2 = tokens[2]
      const dstMem = parseMemOperand(dst)
      const srcMem = parseMemOperand(src2)
      const dstIdx = regIndex(dst)
      const srcIdx = regIndex(src2)

      if (!dstMem && !srcMem) {
        if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${dst}'`); continue }
        if (srcIdx >= 0) {
          out.push(0x8B, 0xC0 | (dstIdx << 3) | srcIdx)
        } else {
          const imm = toNum(src2)
          if (imm == null) { errors.push(`Line ${lineNum}: Expected register or immediate`); continue }
          out.push(0xB8 + dstIdx, imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
        }
      } else if (!dstMem && srcMem) {
        if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${dst}'`); continue }
        if (srcMem.type === 'reg' && srcMem.reg === 4) {
          errors.push(`Line ${lineNum}: [ESP] addressing requires SIB byte (not supported)`); continue
        }
        const rmBytes = encodeRmMem(srcMem, dstIdx)
        if (!rmBytes.length) { errors.push(`Line ${lineNum}: Unsupported memory addressing`); continue }
        out.push(0x8B, ...rmBytes)
      } else if (dstMem && !srcMem) {
        if (srcIdx < 0) { errors.push(`Line ${lineNum}: Expected register source for memory store`); continue }
        if (dstMem.type === 'reg' && dstMem.reg === 4) {
          errors.push(`Line ${lineNum}: [ESP] addressing requires SIB byte (not supported)`); continue
        }
        const rmBytes = encodeRmMem(dstMem, srcIdx)
        if (!rmBytes.length) { errors.push(`Line ${lineNum}: Unsupported memory addressing`); continue }
        out.push(0x89, ...rmBytes)
      } else {
        errors.push(`Line ${lineNum}: MOV does not support memory-to-memory`); continue
      }

    // ── PUSH ─────────────────────────────────────────────────────────────────
    } else if (op === 'PUSH') {
      if (tokens.length !== 2) { errors.push(`Line ${lineNum}: PUSH expects 1 operand`); continue }
      const idx = regIndex(tokens[1])
      if (idx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      out.push(0x50 + idx)

    // ── POP ──────────────────────────────────────────────────────────────────
    } else if (op === 'POP') {
      if (tokens.length !== 2) { errors.push(`Line ${lineNum}: POP expects 1 operand`); continue }
      const idx = regIndex(tokens[1])
      if (idx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      out.push(0x58 + idx)

    // ── ADD ──────────────────────────────────────────────────────────────────
    } else if (op === 'ADD') {
      if (tokens.length !== 3) { errors.push(`Line ${lineNum}: ADD expects 2 operands`); continue }
      const dstIdx = regIndex(tokens[1])
      if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      const srcIdx = regIndex(tokens[2])
      if (srcIdx >= 0) {
        out.push(0x01, 0xC0 | (srcIdx << 3) | dstIdx)
      } else {
        const imm = toNum(tokens[2])
        if (imm == null) { errors.push(`Line ${lineNum}: Expected register or immediate`); continue }
        out.push(0x81, 0xC0 | dstIdx, imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
      }

    // ── SUB ──────────────────────────────────────────────────────────────────
    } else if (op === 'SUB') {
      if (tokens.length !== 3) { errors.push(`Line ${lineNum}: SUB expects 2 operands`); continue }
      const dstIdx = regIndex(tokens[1])
      if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      const srcIdx = regIndex(tokens[2])
      if (srcIdx >= 0) {
        out.push(0x29, 0xC0 | (srcIdx << 3) | dstIdx)
      } else {
        const imm = toNum(tokens[2])
        if (imm == null) { errors.push(`Line ${lineNum}: Expected register or immediate`); continue }
        out.push(0x81, 0xE8 | dstIdx, imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
      }

    // ── CMP ──────────────────────────────────────────────────────────────────
    } else if (op === 'CMP') {
      if (tokens.length !== 3) { errors.push(`Line ${lineNum}: CMP expects 2 operands`); continue }
      const dstIdx = regIndex(tokens[1])
      if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      const srcIdx = regIndex(tokens[2])
      if (srcIdx >= 0) {
        // CMP r/m32, r32: 0x39
        out.push(0x39, 0xC0 | (srcIdx << 3) | dstIdx)
      } else {
        const imm = toNum(tokens[2])
        if (imm == null) { errors.push(`Line ${lineNum}: Expected register or immediate`); continue }
        // CMP r/m32, imm32: 0x81 /7
        out.push(0x81, 0xC0 | (7 << 3) | dstIdx, imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
      }

    // ── AND ──────────────────────────────────────────────────────────────────
    } else if (op === 'AND') {
      if (tokens.length !== 3) { errors.push(`Line ${lineNum}: AND expects 2 operands`); continue }
      const dstIdx = regIndex(tokens[1])
      if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      const srcIdx = regIndex(tokens[2])
      if (srcIdx >= 0) {
        out.push(0x21, 0xC0 | (srcIdx << 3) | dstIdx)
      } else {
        const imm = toNum(tokens[2])
        if (imm == null) { errors.push(`Line ${lineNum}: Expected register or immediate`); continue }
        out.push(0x81, 0xC0 | (4 << 3) | dstIdx, imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
      }

    // ── OR ───────────────────────────────────────────────────────────────────
    } else if (op === 'OR') {
      if (tokens.length !== 3) { errors.push(`Line ${lineNum}: OR expects 2 operands`); continue }
      const dstIdx = regIndex(tokens[1])
      if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      const srcIdx = regIndex(tokens[2])
      if (srcIdx >= 0) {
        out.push(0x09, 0xC0 | (srcIdx << 3) | dstIdx)
      } else {
        const imm = toNum(tokens[2])
        if (imm == null) { errors.push(`Line ${lineNum}: Expected register or immediate`); continue }
        out.push(0x81, 0xC0 | (1 << 3) | dstIdx, imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
      }

    // ── SHL ──────────────────────────────────────────────────────────────────
    } else if (op === 'SHL' || op === 'SAL') {
      if (tokens.length !== 3) { errors.push(`Line ${lineNum}: SHL expects 2 operands`); continue }
      const dstIdx = regIndex(tokens[1])
      if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      const count = toNum(tokens[2])
      if (count == null) { errors.push(`Line ${lineNum}: Expected immediate count`); continue }
      out.push(0xC1, 0xC0 | (4 << 3) | dstIdx, count & 0xFF)

    // ── SHR ──────────────────────────────────────────────────────────────────
    } else if (op === 'SHR') {
      if (tokens.length !== 3) { errors.push(`Line ${lineNum}: SHR expects 2 operands`); continue }
      const dstIdx = regIndex(tokens[1])
      if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      const count = toNum(tokens[2])
      if (count == null) { errors.push(`Line ${lineNum}: Expected immediate count`); continue }
      out.push(0xC1, 0xC0 | (5 << 3) | dstIdx, count & 0xFF)

    // ── SAR ──────────────────────────────────────────────────────────────────
    } else if (op === 'SAR') {
      if (tokens.length !== 3) { errors.push(`Line ${lineNum}: SAR expects 2 operands`); continue }
      const dstIdx = regIndex(tokens[1])
      if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      const count = toNum(tokens[2])
      if (count == null) { errors.push(`Line ${lineNum}: Expected immediate count`); continue }
      out.push(0xC1, 0xC0 | (7 << 3) | dstIdx, count & 0xFF)

    // ── MUL ──────────────────────────────────────────────────────────────────
    } else if (op === 'MUL') {
      if (tokens.length !== 2) { errors.push(`Line ${lineNum}: MUL expects 1 register operand`); continue }
      const srcIdx = regIndex(tokens[1])
      if (srcIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      out.push(0xF7, 0xC0 | (4 << 3) | srcIdx)

    // ── IDIV ─────────────────────────────────────────────────────────────────
    } else if (op === 'IDIV') {
      if (tokens.length !== 2) { errors.push(`Line ${lineNum}: IDIV expects 1 register operand`); continue }
      const srcIdx = regIndex(tokens[1])
      if (srcIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      out.push(0xF7, 0xC0 | (7 << 3) | srcIdx)

    // ── IMUL (two-operand: IMUL reg, reg) ────────────────────────────────────
    } else if (op === 'IMUL') {
      if (tokens.length !== 3) { errors.push(`Line ${lineNum}: IMUL expects 2 register operands`); continue }
      const dstIdx = regIndex(tokens[1])
      const srcIdx2 = regIndex(tokens[2])
      if (dstIdx < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[1]}'`); continue }
      if (srcIdx2 < 0) { errors.push(`Line ${lineNum}: Unknown register '${tokens[2]}'`); continue }
      out.push(0x0F, 0xAF, 0xC0 | (dstIdx << 3) | srcIdx2)

    // ── CDQ ──────────────────────────────────────────────────────────────────
    } else if (op === 'CDQ') {
      out.push(0x99)

    // ── JMP ──────────────────────────────────────────────────────────────────
    } else if (op === 'JMP') {
      if (tokens.length !== 2) { errors.push(`Line ${lineNum}: JMP expects 1 operand`); continue }
      const target = tokens[1].toUpperCase()
      const labelAddr = labels.get(target)
      if (labelAddr !== undefined) {
        // Label target: compute relative displacement (rel8 from end of 2-byte instruction)
        const instrEnd = currentOffset + 2
        const rel = labelAddr - instrEnd
        if (rel < -128 || rel > 127) {
          errors.push(`Line ${lineNum}: JMP target '${target}' is too far for rel8 (${rel})`); continue
        }
        out.push(0xEB, rel & 0xFF)
      } else {
        const rel = toNum(tokens[1])
        if (rel == null) { errors.push(`Line ${lineNum}: Unknown label or invalid displacement '${tokens[1]}'`); continue }
        const relS = rel | 0
        if (relS >= -128 && relS <= 127) {
          out.push(0xEB, relS & 0xFF)
        } else {
          out.push(0xE9, rel & 0xFF, (rel >>> 8) & 0xFF, (rel >>> 16) & 0xFF, (rel >>> 24) & 0xFF)
        }
      }

    // ── Conditional Jumps (JE, JNE, JL, JGE, JLE, JG, etc.) ─────────────────
    } else if (op in JCC_OPCODES) {
      if (tokens.length !== 2) { errors.push(`Line ${lineNum}: ${op} expects 1 operand`); continue }
      const jccOpcode = JCC_OPCODES[op]
      const target = tokens[1].toUpperCase()
      const labelAddr = labels.get(target)
      if (labelAddr !== undefined) {
        const instrEnd = currentOffset + 2
        const rel = labelAddr - instrEnd
        if (rel < -128 || rel > 127) {
          errors.push(`Line ${lineNum}: ${op} target '${target}' is too far for rel8 (${rel})`); continue
        }
        out.push(jccOpcode, rel & 0xFF)
      } else {
        const rel = toNum(tokens[1])
        if (rel == null) { errors.push(`Line ${lineNum}: Unknown label or invalid displacement '${tokens[1]}'`); continue }
        const relS = rel | 0
        if (relS < -128 || relS > 127) {
          errors.push(`Line ${lineNum}: ${op} displacement out of rel8 range (${relS})`); continue
        }
        out.push(jccOpcode, relS & 0xFF)
      }

    // ── CALL ─────────────────────────────────────────────────────────────────
    } else if (op === 'CALL') {
      if (tokens.length !== 2) { errors.push(`Line ${lineNum}: CALL expects 1 operand`); continue }
      const target = tokens[1].toUpperCase()
      const labelAddr = labels.get(target)
      if (labelAddr !== undefined) {
        // Label target: compute rel32 displacement from end of 5-byte CALL instruction
        const instrEnd = currentOffset + 5
        const rel = labelAddr - instrEnd
        const r = rel >>> 0
        out.push(0xE8, r & 0xFF, (r >>> 8) & 0xFF, (r >>> 16) & 0xFF, (r >>> 24) & 0xFF)
      } else {
        const rel = toNum(tokens[1])
        if (rel == null) { errors.push(`Line ${lineNum}: Unknown label or invalid displacement '${tokens[1]}'`); continue }
        out.push(0xE8, rel & 0xFF, (rel >>> 8) & 0xFF, (rel >>> 16) & 0xFF, (rel >>> 24) & 0xFF)
      }

    // ── RET ──────────────────────────────────────────────────────────────────
    } else if (op === 'RET') {
      if (tokens.length !== 1) { errors.push(`Line ${lineNum}: RET takes no operands`); continue }
      out.push(0xC3)

    } else {
      errors.push(`Line ${lineNum}: Unknown or unsupported mnemonic '${op}'`)
    }
  }

  return { bytes: new Uint8Array(out), errors }
}

// ─── Grading is handled by the Rust WASM module (core/src/grader/) ────────────

// ─── LabPage component ────────────────────────────────────────────────────────

export default function LabPage() {
  const location = useLocation()
  const navigate = useNavigate()
  const labNum = parseInt(location.pathname.replace('/lab', '')) || 1
  const config = labConfigs[labNum] ?? labConfigs[1]

  const [code, setCode] = useState(config.starterCode)
  const [consoleOutput, setConsoleOutput] = useState('')
  const [steps, setSteps] = useState(0)
  const [wasmReady, setWasmReady] = useState(false)
  const [userRole, setUserRole] = useState<string | null>(null)
  const [username, setUsername] = useState<string | null>(null)

  const wasmEmuRef = useRef<EmulatorApi | null>(null)
  const wasmModRef = useRef<WasmModule | null>(null)

  const [registers, setRegisters] = useState({
    eip: '0x00001000',
    eax: '0x00000000',
    ebx: '0x00000000',
    ecx: '0x00000000',
    edx: '0x00000000',
    ebp: '0x00F00000',
    esp: '0x00F00000',
    esi: '0x00000000',
    edi: '0x00000000',
  })

  const [flags, setFlags] = useState({ zf: 0, sf: 0, of: 0, cf: 0, df: 0, pf: 0 })
  const [memoryView, setMemoryView] = useState<number[]>(Array(48).fill(0))

  // Check auth and role on mount
  useEffect(() => {
    const role = localStorage.getItem('userRole')
    const user = localStorage.getItem('username')

    if (!role) {
      navigate('/login')
      return
    }

    setUserRole(role)
    setUsername(user || 'User')
  }, [navigate])

  // Reset state when switching labs
  useEffect(() => {
    setCode(config.starterCode)
    setConsoleOutput('')
    setSteps(0)
    setRegisters({
      eip: '0x00001000', eax: '0x00000000', ebx: '0x00000000',
      ecx: '0x00000000', edx: '0x00000000', ebp: '0x00F00000',
      esp: '0x00F00000', esi: '0x00000000', edi: '0x00000000',
    })
    setFlags({ zf: 0, sf: 0, of: 0, cf: 0, df: 0, pf: 0 })
    setMemoryView(Array(48).fill(0))
    if (wasmEmuRef.current) {
      try { wasmEmuRef.current.reset() } catch (_) { /* ignore */ }
      wasmEmuRef.current = null
    }
  }, [labNum]) // eslint-disable-line react-hooks/exhaustive-deps

  // Load WASM once on mount
  useEffect(() => {
    let mounted = true
    ;(async () => {
      try {
        const wasm: WasmModule = await import('./wasm/pkg/web_x86_core')
        await wasm.default()
        wasmModRef.current = wasm
        if (mounted) {
          setConsoleOutput('WASM: module ready\n')
          setWasmReady(true)
        }
      } catch (err) {
        if (mounted) setConsoleOutput(`WASM load error: ${String(err)}\n`)
      }
    })()
    return () => { mounted = false }
  }, [])

  function refreshRegistersFromWasm(emu: EmulatorApi) {
    const fmt = (n: number | bigint) => {
      const v = typeof n === 'bigint' ? Number(n) : n
      return `0x${(v >>> 0).toString(16).padStart(8, '0').toUpperCase()}`
    }
    setRegisters({
      eip: fmt(emu.get_eip()), eax: fmt(emu.get_eax()), ebx: fmt(emu.get_ebx()),
      ecx: fmt(emu.get_ecx()), edx: fmt(emu.get_edx()), ebp: fmt(emu.get_ebp()),
      esp: fmt(emu.get_esp()), esi: fmt(emu.get_esi()), edi: fmt(emu.get_edi()),
    })
    setFlags({
      zf: emu.get_zf() ? 1 : 0,
      sf: emu.get_sf() ? 1 : 0,
      of: emu.get_of() ? 1 : 0,
      cf: emu.get_cf() ? 1 : 0,
      pf: emu.get_pf() ? 1 : 0,
      df: 0,
    })
    try {
      setMemoryView(Array.from({ length: 48 }, (_, i) => Number(emu.read_u8(LOAD_ADDR + i)) & 0xFF))
    } catch (_) { /* memory read optional */ }
  }

  function onRun() {
    if (!wasmReady || !wasmModRef.current) {
      setConsoleOutput((s) => s + 'WASM not ready\n')
      return
    }
    const { Emulator } = wasmModRef.current
    const emu = new Emulator()
    wasmEmuRef.current = emu

    const { bytes, errors } = assemble(code)
    if (errors.length) {
      setConsoleOutput((s) => s + errors.map((e) => `ASM error: ${e}`).join('\n') + '\n')
      return
    }
    try {
      emu.load_program(bytes, LOAD_ADDR)
      setConsoleOutput((s) => s + `Assembled ${bytes.length} bytes. Running...\n`)
      for (let i = 0; i < 1000; i++) emu.step()
      const total = Number(emu.get_steps?.() ?? 0)
      setSteps(total)
      refreshRegistersFromWasm(emu)
      setConsoleOutput((s) => s + `Run complete. Steps: ${total}\n`)
    } catch (e) {
      setConsoleOutput((s) => s + `Runtime error: ${String(e)}\n`)
    }
  }

  function onStep() {
    if (!wasmReady || !wasmModRef.current) {
      setConsoleOutput((s) => s + 'WASM not ready\n')
      return
    }
    if (!wasmEmuRef.current) {
      const { Emulator } = wasmModRef.current
      const emu = new Emulator()
      wasmEmuRef.current = emu
      const { bytes, errors } = assemble(code)
      if (errors.length) {
        setConsoleOutput((s) => s + errors.map((e) => `ASM error: ${e}`).join('\n') + '\n')
        return
      }
      try {
        emu.load_program(bytes, LOAD_ADDR)
        setConsoleOutput((s) => s + `Assembled ${bytes.length} bytes. Stepping...\n`)
      } catch (e) {
        setConsoleOutput((s) => s + `Load error: ${String(e)}\n`)
        return
      }
    }
    try {
      wasmEmuRef.current.step()
      const n = Number(wasmEmuRef.current.get_steps())
      setSteps(n)
      refreshRegistersFromWasm(wasmEmuRef.current)
      setConsoleOutput((s) => s + `Step ${n}: EIP=${registers.eip}\n`)
    } catch (e) {
      setConsoleOutput((s) => s + `Step error: ${String(e)}\n`)
    }
  }

  function onReset() {
    if (wasmEmuRef.current) {
      try { wasmEmuRef.current.reset() } catch (_) { /* ignore */ }
      wasmEmuRef.current = null
    }
    setSteps(0)
    setConsoleOutput('')
    setRegisters({
      eip: '0x00001000', eax: '0x00000000', ebx: '0x00000000',
      ecx: '0x00000000', edx: '0x00000000', ebp: '0x00F00000',
      esp: '0x00F00000', esi: '0x00000000', edi: '0x00000000',
    })
    setFlags({ zf: 0, sf: 0, of: 0, cf: 0, df: 0, pf: 0 })
    setMemoryView(Array(48).fill(0))
  }

  function buildGradingResult(): GradingResult | null {
    if (!wasmModRef.current) return null

    const { bytes, errors } = assemble(code)
    if (errors.length) {
      return {
        earned: 0,
        total: 0,
        autoMax: 0,
        details: ['Assembly errors – fix your code before submitting:', ...errors],
      }
    }

    try {
      // grade_lab is exposed by the Rust grader module via wasm_bindgen.
      // Type will be available after wasm-pack rebuild.
      const gradeLab = (wasmModRef.current as any).grade_lab as (lab: number, program: Uint8Array) => string
      const json = gradeLab(labNum, bytes)
      return JSON.parse(json) as GradingResult
    } catch (e) {
      return {
        earned: 0,
        total: 0,
        autoMax: 0,
        details: [`Grading error: ${String(e)}`],
      }
    }
  }

  function handleStudentSubmit() {
    const result = buildGradingResult()
    if (!result) return null

    if (userRole === 'student') {
      saveStudentSubmission({
        labId: labNum,
        studentUsername: username || 'Unknown Student',
        autoEarned: result.earned,
        total: result.total,
        details: result.details,
      })
    }

    return result
  }

  return (
    <div className="app-root">
      <header className="topbar">
        <div className="brand">ASU</div>
        <div className="title">{config.title}</div>
        <div style={{ marginLeft: 'auto', paddingRight: '1rem', display: 'flex', alignItems: 'center', gap: '1rem', fontSize: '0.9rem' }}>
          <span>
            {userRole === 'admin' ? 'Instructor/Admin' : 'Student'}: {username}
          </span>
        </div>
        <div className="toolbar">
          <button onClick={onRun} className="primary">Run</button>
          <button onClick={onStep}>Step</button>
          <button onClick={onReset}>Reset</button>
        </div>
      </header>

      <main className="main-grid">
        {/* Assembly Editor */}
        <section className="editor-pane">
          <div className="editor-header">Assembly Editor</div>
          <textarea
            className="editor"
            value={code}
            onChange={(e) => setCode(e.target.value)}
            spellCheck={false}
            aria-label="Assembly editor"
          />
        </section>

        {/* Right Panel */}
        <aside className="sidebar">
          <p className="steps-counter">Steps: {steps}</p>

          <div className="panel-heading">Registers</div>
          <div className="registers">
            {(['eip','eax','ebx','ecx','edx','ebp','esp','esi','edi'] as const).map((r) => (
              <div key={r} className="reg-row">
                <span className="reg-name">{r.toUpperCase()}</span>
                <span className="reg-val">{registers[r]}</span>
              </div>
            ))}
          </div>

          <div className="panel-heading" style={{ marginTop: 12 }}>Flags</div>
          <div className="registers">
            {(['zf','sf','of','cf','df','pf'] as const).map((f) => (
              <div key={f} className="reg-row">
                <span className="reg-name">{f.toUpperCase()}</span>
                <span className={`reg-val flag-val${flags[f] ? ' flag-set' : ''}`}>{flags[f]}</span>
              </div>
            ))}
          </div>

          <div className="panel-heading" style={{ marginTop: 12 }}>
            Memory <span className="mem-addr-label">@ 0x1000</span>
          </div>
          <div className="memory-grid">
            {memoryView.map((b, i) => (
              <div key={i} className="mem-cell">
                {b.toString(16).toUpperCase().padStart(2, '0')}
              </div>
            ))}
          </div>

          <GradingPanel
            labId={labNum}
            description={config.description}
            onSubmit={handleStudentSubmit}
          />
        </aside>

        {/* Console */}
        <section className="console-pane">
          <div className="console-header">Console</div>
          <pre className="console-output" role="log" aria-live="polite">{consoleOutput}</pre>
        </section>
      </main>
    </div>
  )
}
