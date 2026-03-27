import { useState, useEffect, useRef } from 'react'
import './App.css'

// Types for the generated WASM bindings
type WasmModule = typeof import('./wasm/pkg/web_x86_core')
type EmulatorApi = import('./wasm/pkg/web_x86_core').Emulator

const SAMPLE_CODE = `; Demo: MOV, PUSH, POP, SUB, and ADD
MOV EAX, 0x100
MOV EBX, 0x50
ADD EAX, EBX
SUB EAX, 0x30
PUSH EAX
POP ECX
; After Run, check:
; - EAX = 0x00000120 (0x100 + 0x50 - 0x30)
; - ECX = 0x00000120 (popped from stack)
; - EBX = 0x00000050 (unchanged)`

const REGISTER_KEYS = ['eip', 'eax', 'ebx', 'ecx', 'edx', 'ebp', 'esp', 'esi', 'edi'] as const
type RegisterKey = (typeof REGISTER_KEYS)[number]
type RegistersState = Record<RegisterKey, string>

const DEFAULT_REGISTERS: RegistersState = {
  eip: '0x00001000',
  eax: '0x00000078',
  ebx: '0x00000000',
  ecx: '0x00000000',
  edx: '0x00000000',
  ebp: '0x00FF0000',
  esp: '0x00FF0000',
  esi: '0x00000000',
  edi: '0x00000000',
}

export default function App() {
//zoom feature
  const EDITOR_BASE_FONT_SIZE = 13
  const MIN_EDITOR_ZOOM = 10
  const MAX_EDITOR_ZOOM = 300
  const EDITOR_ZOOM_STEP = 10
  
  const [code, setCode] = useState(SAMPLE_CODE)
  const [consoleOutput, setConsoleOutput] = useState('Hello, World!\n')
  const [steps, setSteps] = useState(0)
  const [wasmReady, setWasmReady] = useState(false)
  const [editorZoom, setEditorZoom] = useState(100)
  const wasmEmuRef = useRef<EmulatorApi | null>(null)
  const wasmModRef = useRef<WasmModule | null>(null)
  const LOAD_ADDR = 0x00001000

  const editorFontSize = Math.round((EDITOR_BASE_FONT_SIZE * editorZoom) / 100)
  const zoomInEditor = () => {
    setEditorZoom((z) => Math.min(MAX_EDITOR_ZOOM, z + EDITOR_ZOOM_STEP))
  }
  const zoomOutEditor = () => {
    setEditorZoom((z) => Math.max(MIN_EDITOR_ZOOM, z - EDITOR_ZOOM_STEP))
  }
  // placeholder registers
  const [registers, setRegisters] = useState<RegistersState>({ ...DEFAULT_REGISTERS })
  const lastValidRegistersRef = useRef<RegistersState>({ ...DEFAULT_REGISTERS })
  const warnedUnsupportedRegsRef = useRef<Set<RegisterKey>>(new Set())

  // placeholder flags
  const [flags, setFlags] = useState({
    zf: 1,
    sf: 0,
    of: 0,
    cf: 0,
    df: 0,
    pf: 1,
  })

  // Memory view (visualization grid). 48 bytes (6 rows × 8 cols) to match mockup.
  const [memoryView, setMemoryView] = useState<number[]>(Array(48).fill(0))

  const setRegistersCommitted = (next: RegistersState) => {
    const committed = { ...next }
    lastValidRegistersRef.current = committed
    setRegisters(committed)
  }

  const parseRegisterValue = (raw: string): number | null => {
    const t = raw.trim()
    if (!t) return null
    if (/^0x[0-9a-f]+$/i.test(t)) {
      return parseInt(t, 16) >>> 0
    }
    if (/^[+-]?\d+$/.test(t)) {
      return parseInt(t, 10) >>> 0
    }
    if (/^[0-9a-f]+$/i.test(t) && /[a-f]/i.test(t)) {
      return parseInt(t, 16) >>> 0
    }
    return null
  }

  const formatRegisterValue = (n: number | bigint) => {
    const val = typeof n === 'bigint' ? Number(n) : n
    return `0x${(val >>> 0).toString(16).padStart(8, '0')}`
  }

  const setEmuRegister = (emu: EmulatorApi, reg: RegisterKey, value: number): boolean => {
    if (reg === 'eax') {
      emu.set_eax(value)
      return true
    }
    return false
  }

  const applyRegistersToEmu = (emu: EmulatorApi, regs: RegistersState) => {
    for (const reg of REGISTER_KEYS) {
      const parsed = parseRegisterValue(regs[reg])
      if (parsed == null) continue
      setEmuRegister(emu, reg, parsed)
    }
  }

  const commitRegister = (reg: RegisterKey) => {
    const parsed = parseRegisterValue(registers[reg])
    if (parsed == null) {
      setConsoleOutput((s) => s + `Invalid ${reg.toUpperCase()} value: ${registers[reg]}\n`)
      setRegisters({ ...lastValidRegistersRef.current })
      return
    }
    const formatted = formatRegisterValue(parsed)
    const next = { ...lastValidRegistersRef.current, [reg]: formatted }
    setRegistersCommitted(next)

    const emu = wasmEmuRef.current
    if (emu) {
      const didSet = setEmuRegister(emu, reg, parsed)
      if (!didSet && !warnedUnsupportedRegsRef.current.has(reg)) {
        warnedUnsupportedRegsRef.current.add(reg)
        setConsoleOutput((s) => s + `Note: ${reg.toUpperCase()} cannot be pushed to emulator yet; backend currently exposes set_eax only.\n`)
      }
    }
  }

  const onRegisterInputChange = (reg: RegisterKey, value: string) => {
    setRegisters((prev) => ({ ...prev, [reg]: value }))
  }

  // 1. cd core   /   wasm-pack build --target web --out-dir ../frontend/src/wasm/pkg --dev --out-name web_x86_cor
  // 2. cd frontend   /   npm install   /   npm run dev
  useEffect(() => {
    let mounted = true
    ;(async () => {
      try {
        const wasm: WasmModule = await import('./wasm/pkg/web_x86_core')
        // Initialize the WASM module (default export is the init function)
        await wasm.default()
        // Only preload module; instantiate Emulator later on Run (frontend controls lifecycle)
        wasmModRef.current = wasm

        if (mounted) {
          setConsoleOutput((s) => s + 'WASM: module ready\n')
          setWasmReady(true)
        }
      } catch (err) {
        console.error('WASM load error', err)
        setConsoleOutput((s) => s + `WASM load error: ${String(err)}\n`)
      }
    })()
    return () => {
      mounted = false
    }
  }, [])

  // Minimal assembler: supports
  // - MOV <REG>, <IMM32>   (encodes B8..BF + imm32)
  // - PUSH <REG>           (encodes 50..57)
  // - POP <REG>            (encodes 58..5F)
  // - ADD <REG>, <REG|IMM32> (01/81 /0)
  // - SUB <REG>, <REG|IMM32> (29/81 /5)
  // - CMP <REG>, <REG|IMM32> (3B/81 /7)
  // - JMP <REL|LABEL>      (EB rel8 if -128..127 else E9 rel32)
  // - RET                  (C3)
  // Lines can contain comments starting with ';'
  function assemble(src: string): { bytes: Uint8Array; errors: string[] } {
    const out: number[] = []
    const errors: string[] = []

    const toNum = (tok: string): number | null => {
      const t = tok.trim()
      if (/^0x[0-9a-f]+$/i.test(t)) {
        return parseInt(t, 16) >>> 0
      }
      if (/^[+-]?\d+$/.test(t)) {
        return (parseInt(t, 10) >>> 0)
      }
      return null
    }

    const regIndex = (r: string): number => {
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

    const stripComment = (raw: string): string => raw.split(';')[0].trim()

    const splitLabels = (line: string): { labels: string[]; body: string } => {
      const labels: string[] = []
      let body = line.trim()
      while (true) {
        const m = body.match(/^([A-Za-z_.$][A-Za-z0-9_.$]*):/)
        if (!m) break
        labels.push(m[1].toUpperCase())
        body = body.slice(m[0].length).trim()
      }
      return { labels, body }
    }

    const tokenize = (line: string): string[] =>
      line
        .replace(/\s+/g, ' ')
        .replace(/\s*,\s*/g, ',')
        .trim()
        .split(/[\s,]/)
        .filter(Boolean)

    const estimateLength = (parts: string[]): number => {
      if (parts.length === 0) return 0
      const op = parts[0].toUpperCase()

      if (op === 'MOV') {
        if (parts.length !== 3 || regIndex(parts[1]) < 0) return 0
        return regIndex(parts[2]) >= 0 ? 2 : 5
      }

      if (op === 'PUSH' || op === 'POP') {
        return parts.length === 2 ? 1 : 0
      }

      if (op === 'ADD' || op === 'SUB' || op === 'CMP') {
        if (parts.length !== 3 || regIndex(parts[1]) < 0) return 0
        return regIndex(parts[2]) >= 0 ? 2 : 6
      }

      if (op === 'JMP') {
        if (parts.length !== 2) return 0
        const rel = toNum(parts[1])
        if (rel == null) return 5 // label-based JMP is encoded as near rel32
        return rel >= -128 && rel <= 127 ? 2 : 5
      }

      if (op === 'RET') {
        return parts.length === 1 ? 1 : 0
      }

      return 0
    }

    const lines = src.split('\n')
    const labels = new Map<string, number>()

    // First pass: resolve label offsets
    let pc = 0
    for (let i = 0; i < lines.length; i++) {
      const raw = lines[i]
      const stripped = stripComment(raw)
      if (!stripped) continue

      const { labels: lineLabels, body } = splitLabels(stripped)
      for (const label of lineLabels) {
        if (labels.has(label)) {
          errors.push(`Line ${i + 1}: Duplicate label '${label}'`)
          continue
        }
        labels.set(label, pc)
      }

      if (!body) continue
      if (/^(section|db|dw|dd)\b/i.test(body)) continue

      const parts = tokenize(body)
      pc += estimateLength(parts)
    }

    // Second pass: encode instructions
    for (let i = 0; i < lines.length; i++) {
      const raw = lines[i]
      const stripped = stripComment(raw)
      if (!stripped) continue

      const { body } = splitLabels(stripped)
      if (!body) continue

      // Ignore very basic data/section directives for now
      if (/^(section|db|dw|dd)\b/i.test(body)) {
        continue
      }

      const parts = tokenize(body)

      const op = parts[0].toUpperCase()
      if (op === 'MOV') {
        if (parts.length !== 3) {
          errors.push(`Line ${i + 1}: MOV expects 2 operands`)
          continue
        }
        const dst = parts[1].toUpperCase()
        const dstIdx = regIndex(dst)
        if (dstIdx < 0) {
          errors.push(`Line ${i + 1}: Unsupported register '${dst}'`)
          continue
        }
        const src = parts[2].toUpperCase()
        const srcIdx = regIndex(src)
        if (srcIdx >= 0) {
          // MOV r/m32, r32 (register-to-register)
          const modrm = 0xC0 | (srcIdx << 3) | dstIdx
          out.push(0x89, modrm)
          continue
        }
        const imm = toNum(parts[2])
        if (imm == null) {
          errors.push(`Line ${i + 1}: Expected immediate or register`)
          continue
        }
        // B8..BF + imm32 (little-endian)
        out.push(0xB8 + dstIdx)
        out.push(imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
      } else if (op === 'PUSH') {
        if (parts.length !== 2) {
          errors.push(`Line ${i + 1}: PUSH expects 1 register operand`)
          continue
        }
        const r = parts[1].toUpperCase()
        const idx = regIndex(r)
        if (idx < 0) {
          errors.push(`Line ${i + 1}: Unsupported register '${r}'`)
          continue
        }
        out.push(0x50 + idx)
      } else if (op === 'JMP') {
        if (parts.length !== 2) {
          errors.push(`Line ${i + 1}: JMP expects 1 operand`)
          continue
        }
        const relToken = parts[1]
        const rel = toNum(relToken)
        if (rel != null) {
          // Numeric displacement path.
          if (rel >= -128 && rel <= 127) {
            out.push(0xEB, (rel & 0xFF) >>> 0)
          } else {
            out.push(0xE9, rel & 0xFF, (rel >>> 8) & 0xFF, (rel >>> 16) & 0xFF, (rel >>> 24) & 0xFF)
          }
        } else {
          const target = labels.get(relToken.toUpperCase())
          if (target == null) {
            errors.push(`Line ${i + 1}: Unknown label '${relToken}'`)
            continue
          }
          // Label path: encode near rel32 displacement from next EIP.
          const displacement = target - (out.length + 5)
          out.push(
            0xE9,
            displacement & 0xFF,
            (displacement >>> 8) & 0xFF,
            (displacement >>> 16) & 0xFF,
            (displacement >>> 24) & 0xFF,
          )
        }
      } else if (op === 'POP') {
        if (parts.length !== 2) {
          errors.push(`Line ${i + 1}: POP expects 1 register operand`)
          continue
        }
        const r = parts[1].toUpperCase()
        const idx = regIndex(r)
        if (idx < 0) {
          errors.push(`Line ${i + 1}: Unsupported register '${r}'`)
          continue
        }
        // POP register: opcodes 0x58..0x5F
        out.push(0x58 + idx)
      } else if (op === 'SUB') {
        if (parts.length !== 3) {
          errors.push(`Line ${i + 1}: SUB expects 2 operands`)
          continue
        }
        const dst = parts[1].toUpperCase()
        const dstIdx = regIndex(dst)
        if (dstIdx < 0) {
          errors.push(`Line ${i + 1}: Unsupported destination register '${dst}'`)
          continue
        }
        
        // Check if source is register or immediate
        const srcReg = parts[2].toUpperCase()
        const srcIdx = regIndex(srcReg)
        
        if (srcIdx >= 0) {
          // SUB reg, reg: opcode 0x29 + ModR/M byte
          // ModR/M: 11 dst src (both in register mode)
          const modrm = 0xC0 | (srcIdx << 3) | dstIdx
          out.push(0x29, modrm)
        } else {
          // SUB reg, imm: opcode 0x81 + ModR/M byte (reg field = 5 for SUB) + imm32
          const imm = toNum(parts[2])
          if (imm == null) {
            errors.push(`Line ${i + 1}: Expected register or immediate (hex like 0x123 or decimal)`)
            continue
          }
          // ModR/M: 11 101 dst (register mode, SUB opcode extension /5)
          const modrm = 0xE8 | dstIdx
          out.push(0x81, modrm, imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
        }
      } else if (op === 'ADD') {
        if (parts.length !== 3) {
          errors.push(`Line ${i + 1}: ADD expects 2 operands`)
          continue
        }
        const dst = parts[1].toUpperCase()
        const dstIdx = regIndex(dst)
        if (dstIdx < 0) {
          errors.push(`Line ${i + 1}: Unsupported destination register '${dst}'`)
          continue
        }
        
        // Check if source is register or immediate
        const srcReg = parts[2].toUpperCase()
        const srcIdx = regIndex(srcReg)
        
        if (srcIdx >= 0) {
          // ADD reg, reg: opcode 0x01 + ModR/M byte
          // ModR/M: 11 src dst (both in register mode)
          const modrm = 0xC0 | (srcIdx << 3) | dstIdx
          out.push(0x01, modrm)
        } else {
          // ADD reg, imm: opcode 0x81 + ModR/M byte (reg field = 0 for ADD) + imm32
          const imm = toNum(parts[2])
          if (imm == null) {
            errors.push(`Line ${i + 1}: Expected register or immediate (hex like 0x123 or decimal)`)
            continue
          }
          // ModR/M: 11 000 dst (register mode, ADD opcode extension /0)
          const modrm = 0xC0 | dstIdx
          out.push(0x81, modrm, imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
        }
      } else if (op === 'AND') {
        if (parts.length !== 3) {
          errors.push(`Line ${i + 1}: AND expects 2 operands`)
          continue
        }
        const dst = parts[1].toUpperCase()
        const dstIdx = regIndex(dst)
        if (dstIdx < 0) {
          errors.push(`Line ${i + 1}: Unsupported destination register '${dst}'`)
          continue
        }

        // Check if source is register or immediate
        const srcReg = parts[2].toUpperCase()
        const srcIdx = regIndex(srcReg)

        if (srcIdx >= 0) {
          // AND reg, reg: opcode 0x21 + ModR/M byte
          // ModR/M: 11 src dst (both in register mode)
          const modrm = 0xC0 | (srcIdx << 3) | dstIdx
          out.push(0x21, modrm)
        } else {
          // AND reg, imm: opcode 0x81 + ModR/M byte (reg field = 4 for AND) + imm32
          const imm = toNum(parts[2])
          if (imm == null) {
            errors.push(`Line ${i + 1}: Expected register or immediate (hex like 0x123 or decimal)`)
            continue
          }
          // ModR/M: 11 100 dst (register mode, AND opcode extension /4)
          const modrm = 0xE0 | dstIdx
          out.push(0x81, modrm, imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
        }
      } else if (op === 'CMP') {
        if (parts.length !== 3) {
          errors.push(`Line ${i + 1}: CMP expects 2 operands`)
          continue
        }
        const dst = parts[1].toUpperCase()
        const dstIdx = regIndex(dst)
        if (dstIdx < 0) {
          errors.push(`Line ${i + 1}: Unsupported destination register '${dst}'`)
          continue
        }

        const srcReg = parts[2].toUpperCase()
        const srcIdx = regIndex(srcReg)

        if (srcIdx >= 0) {
          // CMP reg, reg: use 0x3B (CMP r32, r/m32) to match backend decode
          // ModR/M: 11 dst src
          const modrm = 0xC0 | (dstIdx << 3) | srcIdx
          out.push(0x3B, modrm)
        } else {
          // CMP reg, imm: opcode 0x81 + ModR/M byte (reg field = 7 for CMP) + imm32
          const imm = toNum(parts[2])
          if (imm == null) {
            errors.push(`Line ${i + 1}: Expected register or immediate (hex like 0x123 or decimal)`)
            continue
          }
          // ModR/M: 11 111 dst (register mode, CMP opcode extension /7)
          const modrm = 0xF8 | dstIdx
          out.push(0x81, modrm, imm & 0xFF, (imm >>> 8) & 0xFF, (imm >>> 16) & 0xFF, (imm >>> 24) & 0xFF)
        }
      } else if (op === 'RET') {
        if (parts.length !== 1) {
          errors.push(`Line ${i + 1}: RET expects no operands`)
          continue
        }
        // RET (near return): opcode 0xC3
        out.push(0xC3)
      } else {
        errors.push(`Line ${i + 1}: Unknown or unsupported mnemonic '${op}'`)
      }
    }

    return { bytes: new Uint8Array(out), errors }
  }

  function onRun() {
    if (!wasmReady || !wasmModRef.current) {
      setConsoleOutput((s) => s + 'WASM not ready\n')
      return
    }

    // Instantiate a fresh Emulator when user decides to Run
    const { Emulator } = wasmModRef.current
    const emu = new Emulator()
    wasmEmuRef.current = emu

    const { bytes, errors } = assemble(code)
    if (errors.length) {
      setConsoleOutput((s) => s + errors.map((e) => `ASM error: ${e}`).join('\n') + '\n')
      return
    }

    try {
      // load assembled bytes at LOAD_ADDR
      emu.load_program(bytes, LOAD_ADDR)
      applyRegistersToEmu(emu, registers)
      setConsoleOutput((s) => s + `Assembled ${bytes.length} bytes. Running...\n`)

      // Run up to a small instruction budget to avoid infinite loops
      const MAX_STEPS = 256
      const programEnd = LOAD_ADDR + bytes.length
      for (let i = 0; i < MAX_STEPS; i++) {
        const before = Number(emu.get_eip())
        // Stop once execution leaves the loaded program window
        if (before < LOAD_ADDR || before >= programEnd) break
        emu.step()
        const after = Number(emu.get_eip())
        // Stop if instruction execution made no forward progress
        if (after === before) break
      }
      const total = Number(emu.get_steps?.() ?? 0)
      setSteps(total)
      refreshRegistersFromWasm(emu)
      setConsoleOutput((s) => s + `Run complete. Steps=${total}\n`)
    } catch (e) {
      setConsoleOutput((s) => s + `WASM runtime error: ${String(e)}\n`)
    }
  }

  function onStep() {
    if (!wasmReady || !wasmModRef.current) {
      setConsoleOutput((s) => s + 'WASM not ready\n')
      return
    }

    // Create emulator if it doesn't exist yet
    if (!wasmEmuRef.current) {
      const { Emulator } = wasmModRef.current
      const emu = new Emulator()
      wasmEmuRef.current = emu
      applyRegistersToEmu(emu, registers)
      setConsoleOutput((s) => s + 'Created new emulator instance\n')
    }

    const emu = wasmEmuRef.current
    try {
      emu.step()
      const stepCount = Number(emu.get_steps())
      setSteps(stepCount)
      setConsoleOutput((s) => s + `Step ${stepCount}\n`)
      refreshRegistersFromWasm(emu)
    } catch (e) {
      setConsoleOutput((s) => s + `WASM step error: ${String(e)}\n`)
    }
  }

  function onReset() {
    const emu = wasmEmuRef.current
    if (emu && wasmReady) {
      try {
        emu.reset()
        setSteps(0)
        setConsoleOutput('')
        refreshRegistersFromWasm(emu)
      } catch (e) {
        setConsoleOutput((s) => s + `WASM reset error: ${String(e)}\n`)
      }
    } else {
      setConsoleOutput((s) => s + 'WASM not ready\n')
    }
  }

  function refreshRegistersFromWasm(emu: EmulatorApi) {
    try {
      const eip = formatRegisterValue(emu.get_eip())
      const eax = formatRegisterValue(emu.get_eax())
      const ebx = formatRegisterValue(emu.get_ebx())
      const ecx = formatRegisterValue(emu.get_ecx())
      const edx = formatRegisterValue(emu.get_edx())
      const ebp = formatRegisterValue(emu.get_ebp())
      const esp = formatRegisterValue(emu.get_esp())
      const esi = formatRegisterValue(emu.get_esi())
      const edi = formatRegisterValue(emu.get_edi())

      setRegistersCommitted({ eip, eax, ebx, ecx, edx, ebp, esp, esi, edi })

      const zf = emu.get_zf() ? 1 : 0
      const sf = emu.get_sf() ? 1 : 0
      const of = emu.get_of() ? 1 : 0
      const cf = emu.get_cf() ? 1 : 0
      const pf = emu.get_pf() ? 1 : 0
      // DF is not implemented in core; keep 0 for now
      const df = 0

      // update flags panel
      setFlags({ zf, sf, of, cf, df, pf })

      // Try to read a small memory window starting at LOAD_ADDR if emulator exposes read_u8
      try {
        const emuUnknown = emu as unknown as { read_u8?: (addr: number) => number }
        const readFn = emuUnknown.read_u8
        if (typeof readFn === 'function') {
          const bytes: number[] = []
          for (let i = 0; i < memoryView.length; i++) {
            const v = readFn(LOAD_ADDR + i)
            bytes.push(Number(v) & 0xFF)
          }
          setMemoryView(bytes)
        }
      } catch (e) {
        setConsoleOutput((s) => s + `${String(e)}\n`)
      }
    } catch (e) {
      setConsoleOutput((s) => s + `WASM refresh error: ${String(e)}\n`)
    }
  }

  return (
    <div className="app-root">
      <header className="topbar">
        <div className="brand">ASU</div>
        <div className="title">Online Assembly x86 Emulator</div>
        <div className="toolbar">
          <button>Open</button>
          <button>Save</button>
          <button>Save as</button>
          <button onClick={onRun} className="primary">Run</button>
          <button onClick={onStep}>Step</button>
          <button onClick={onReset}>Reset</button>
        </div>
      </header>

      <main className="main-grid">
        <section className="editor-pane">
        <div className="editor-header">
            <span>Assembly Editor</span>
            <div className="editor-zoom-controls" role="group" aria-label="Assembly editor zoom controls">
              <button
                type="button"
                className="editor-zoom-button"
                onClick={zoomOutEditor}
                disabled={editorZoom <= MIN_EDITOR_ZOOM}
                aria-label="Zoom out assembly editor"
              >
                -
              </button>
              <span className="editor-zoom-value" aria-live="polite">{editorZoom}%</span>
              <button
                type="button"
                className="editor-zoom-button"
                onClick={zoomInEditor}
                disabled={editorZoom >= MAX_EDITOR_ZOOM}
                aria-label="Zoom in assembly editor"
              >
                +
              </button>
            </div>
          </div>
          <textarea
            className="editor"
            value={code}
            onChange={(e) => setCode(e.target.value)}
            spellCheck={false}
            aria-label="Assembly editor"
            style={{ fontSize: `${editorFontSize}px`}}
          />
        </section>

        <aside className="sidebar">
          <p>Steps: {steps}</p>
          <div className="panel-heading">Registers</div>
          <div className="registers">
            <div className="reg-row">
              <span className="reg-name">EIP</span>
              <input
                className="reg-val reg-input"
                value={registers.eip}
                onChange={(e) => onRegisterInputChange('eip', e.target.value)}
                onBlur={() => commitRegister('eip')}
                onFocus={(e) => e.currentTarget.select()}
                onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur() }}
                aria-label="EIP register"
              />
            </div>
            <div className="reg-row">
              <span className="reg-name">EAX</span>
              <input
                className="reg-val reg-input"
                value={registers.eax}
                onChange={(e) => onRegisterInputChange('eax', e.target.value)}
                onBlur={() => commitRegister('eax')}
                onFocus={(e) => e.currentTarget.select()}
                onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur() }}
                aria-label="EAX register"
              />
            </div>
            <div className="reg-row">
              <span className="reg-name">EBX</span>
              <input
                className="reg-val reg-input"
                value={registers.ebx}
                onChange={(e) => onRegisterInputChange('ebx', e.target.value)}
                onBlur={() => commitRegister('ebx')}
                onFocus={(e) => e.currentTarget.select()}
                onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur() }}
                aria-label="EBX register"
              />
            </div>
            <div className="reg-row">
              <span className="reg-name">ECX</span>
              <input
                className="reg-val reg-input"
                value={registers.ecx}
                onChange={(e) => onRegisterInputChange('ecx', e.target.value)}
                onBlur={() => commitRegister('ecx')}
                onFocus={(e) => e.currentTarget.select()}
                onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur() }}
                aria-label="ECX register"
              />
            </div>
            <div className="reg-row">
              <span className="reg-name">EDX</span>
              <input
                className="reg-val reg-input"
                value={registers.edx}
                onChange={(e) => onRegisterInputChange('edx', e.target.value)}
                onBlur={() => commitRegister('edx')}
                onFocus={(e) => e.currentTarget.select()}
                onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur() }}
                aria-label="EDX register"
              />
            </div>
            <div className="reg-row">
              <span className="reg-name">EBP</span>
              <input
                className="reg-val reg-input"
                value={registers.ebp}
                onChange={(e) => onRegisterInputChange('ebp', e.target.value)}
                onBlur={() => commitRegister('ebp')}
                onFocus={(e) => e.currentTarget.select()}
                onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur() }}
                aria-label="EBP register"
              />
            </div>
            <div className="reg-row">
              <span className="reg-name">ESP</span>
              <input
                className="reg-val reg-input"
                value={registers.esp}
                onChange={(e) => onRegisterInputChange('esp', e.target.value)}
                onBlur={() => commitRegister('esp')}
                onFocus={(e) => e.currentTarget.select()}
                onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur() }}
                aria-label="ESP register"
              />
            </div>
            <div className="reg-row">
              <span className="reg-name">ESI</span>
              <input
                className="reg-val reg-input"
                value={registers.esi}
                onChange={(e) => onRegisterInputChange('esi', e.target.value)}
                onBlur={() => commitRegister('esi')}
                onFocus={(e) => e.currentTarget.select()}
                onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur() }}
                aria-label="ESI register"
              />
            </div>
            <div className="reg-row">
              <span className="reg-name">EDI</span>
              <input
                className="reg-val reg-input"
                value={registers.edi}
                onChange={(e) => onRegisterInputChange('edi', e.target.value)}
                onBlur={() => commitRegister('edi')}
                onFocus={(e) => e.currentTarget.select()}
                onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur() }}
                aria-label="EDI register"
              />
            </div>
          </div>

          <div className="panel-heading" style={{ marginTop: 12 }}>Flags</div>
          <div className="registers">
            <div className="reg-row"><span className="reg-name">ZF</span><span className="reg-val">{flags.zf}</span></div>
            <div className="reg-row"><span className="reg-name">SF</span><span className="reg-val">{flags.sf}</span></div>
            <div className="reg-row"><span className="reg-name">0F</span><span className="reg-val">{flags.of}</span></div>
            <div className="reg-row"><span className="reg-name">CF</span><span className="reg-val">{flags.cf}</span></div>
            <div className="reg-row"><span className="reg-name">DF</span><span className="reg-val">{flags.df}</span></div>
            <div className="reg-row"><span className="reg-name">PF</span><span className="reg-val">{flags.pf}</span></div>
          </div>

          <div className="panel-heading" style={{ marginTop: 12 }}>Memory</div>
          <div className="memory-grid" role="grid" aria-label="Memory view" style={{ display: 'grid', gridTemplateColumns: 'repeat(8, 1fr)', gap: 6 }}>
            {memoryView.map((b, i) => {
              const hex = b.toString(16).toUpperCase().padStart(1, '0')
              return (
                <div key={i} className="mem-cell" role="gridcell" aria-label={`Byte ${i}`} style={{
                  border: '1px solid #ddd',
                  borderRadius: 6,
                  padding: '6px 8px',
                  textAlign: 'center',
                  fontFamily: 'monospace',
                  background: '#fff'
                }}>
                  {hex}
                </div>
              )
            })}
          </div>
        </aside>

        <section className="console-pane">
          <div className="console-header">Console</div>
          <pre className="console-output" role="log" aria-live="polite">{consoleOutput}</pre>
        </section>
      </main>
    </div>
  )
}
