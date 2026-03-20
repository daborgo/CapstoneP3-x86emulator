import { useState, useEffect, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
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

export default function App() {
  const [code, setCode] = useState(SAMPLE_CODE)
  const [consoleOutput, setConsoleOutput] = useState('Hello, World!\n')
  const [steps, setSteps] = useState(0)
  const [wasmReady, setWasmReady] = useState(false)
  const [userRole, setUserRole] = useState<string | null>(null)
  const [username, setUsername] = useState<string | null>(null)
  const wasmEmuRef = useRef<EmulatorApi | null>(null)
  const wasmModRef = useRef<WasmModule | null>(null)
  const LOAD_ADDR = 0x00001000
  const fileInputRef = useRef<HTMLInputElement | null>(null)
  const navigate = useNavigate()


  // placeholder registers
  const [registers, setRegisters] = useState({
    eip: '0x00001000',
    eax: '0x00000078',
    ebx: '0x00000000',
    ecx: '0x00000000',
    edx: '0x00000000',
    ebp: '0xFFFF0000',
    esp: '0xFFFF0000',
    esi: '0x00000000',
    edi: '0x00000000',
  })

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
  // - JMP <REL>            (EB rel8 if -128..127 else E9 rel32)
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

    const lines = src.split('\n')
    // First pass: no labels yet; ignore unknown directives
    for (let i = 0; i < lines.length; i++) {
      const raw = lines[i]
      const line = raw.split(';')[0].trim()
      if (!line) continue

      // Ignore very basic data/section directives for now
      if (/^(section|db|dw|dd)\b/i.test(line)) {
        continue
      }

      const parts = line
        .replace(/\s+/g, ' ')
        .replace(/\s*,\s*/g, ',')
        .trim()
        .split(/[\s,]/)
        .filter(Boolean)

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
        const imm = toNum(parts[2])
        if (imm == null) {
          errors.push(`Line ${i + 1}: Expected immediate (hex like 0x123 or decimal)`)
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
          errors.push(`Line ${i + 1}: JMP expects 1 immediate displacement`)
          continue
        }
        const rel = toNum(parts[1])
        if (rel == null) {
          errors.push(`Line ${i + 1}: JMP displacement must be number`)
          continue
        }
        // Encode as EB rel8 if in range, else E9 rel32
        // NOTE: this 'rel' is literal displacement (not label-based) from next EIP
        if (rel >= -128 && rel <= 127) {
          out.push(0xEB, (rel & 0xFF) >>> 0)
        } else {
          out.push(0xE9, rel & 0xFF, (rel >>> 8) & 0xFF, (rel >>> 16) & 0xFF, (rel >>> 24) & 0xFF)
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
      setConsoleOutput((s) => s + `Assembled ${bytes.length} bytes. Running...\n`)

      // Run up to a small instruction budget to avoid infinite loops
      const MAX_STEPS = 256
      for (let i = 0; i < MAX_STEPS; i++) {
        emu.step()
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

function onOpenFileClick() {
  fileInputRef.current?.click();
}

async function onFileSelected(e: React.ChangeEvent<HTMLInputElement>) {
  const file = e.target.files?.[0];
  if (!file) return;

  try {
    const text = await file.text();
    setCode(text); // loads into your Assembly Editor textarea
    setConsoleOutput((s) => s + `Opened file: ${file.name}\n`);
  } catch (err) {
    console.error(err);
    setConsoleOutput((s) => s + `Open file error: ${String(err)}\n`);
  } finally {
    e.target.value = ""; // allows selecting same file again
  }
}

  function refreshRegistersFromWasm(emu: EmulatorApi) {
    try {
      const fmt = (n: number | bigint) => {
        const val = typeof n === 'bigint' ? Number(n) : n
        return `0x${val.toString(16).padStart(8, '0')}`
      }

      const eip = fmt(emu.get_eip())
      const eax = fmt(emu.get_eax())
      const ebx = fmt(emu.get_ebx())
      const ecx = fmt(emu.get_ecx())
      const edx = fmt(emu.get_edx())
      const ebp = fmt(emu.get_ebp())
      const esp = fmt(emu.get_esp())
      const esi = fmt(emu.get_esi())
      const edi = fmt(emu.get_edi())

      setRegisters({ eip, eax, ebx, ecx, edx, ebp, esp, esi, edi })

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
        <div style={{ marginLeft: 'auto', paddingRight: '1rem', display: 'flex', alignItems: 'center', gap: '1rem', fontSize: '0.9rem' }}>
          <span>
            {userRole === 'admin' ? 'Instructor/Admin' : 'Student'}: {username}
          </span>
        </div>
        <div className="toolbar">
          <button onClick={onOpenFileClick}>Open</button>
          <input
          ref={fileInputRef}
          type="file"
          accept=".txt"
          onChange={onFileSelected}
          style={{ display: "none" }}
        />
          <button>Save</button>
          <button>Save as</button>
          <button onClick={onRun} className="primary">Run</button>
          <button onClick={onStep}>Step</button>
          <button onClick={onReset}>Reset</button>
        </div>
      </header>

      <main className="main-grid">
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

        <aside className="sidebar">
          <p>Steps: {steps}</p>
          <div className="panel-heading">Registers</div>
          <div className="registers">
            <div className="reg-row"><span className="reg-name">EIP</span><span className="reg-val">{registers.eip}</span></div>
            <div className="reg-row"><span className="reg-name">EAX</span><span className="reg-val">{registers.eax}</span></div>
            <div className="reg-row"><span className="reg-name">EBX</span><span className="reg-val">{registers.ebx}</span></div>
            <div className="reg-row"><span className="reg-name">ECX</span><span className="reg-val">{registers.ecx}</span></div>
            <div className="reg-row"><span className="reg-name">EDX</span><span className="reg-val">{registers.edx}</span></div>
            <div className="reg-row"><span className="reg-name">EBP</span><span className="reg-val">{registers.ebp}</span></div>
            <div className="reg-row"><span className="reg-name">ESP</span><span className="reg-val">{registers.esp}</span></div>
            <div className="reg-row"><span className="reg-name">ESI</span><span className="reg-val">{registers.esi}</span></div>
            <div className="reg-row"><span className="reg-name">EDI</span><span className="reg-val">{registers.edi}</span></div>
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