// ...existing code...
import { useState, useEffect, useRef } from 'react'
import './App.css'

const SAMPLE_CODE = `section .data
hello:
    db 'Hello world!', 10, 0

section .text
    MOV EAX, hello
    INT 2    ; print string EAX
    HLT`

export default function App() {
  const [code, setCode] = useState(SAMPLE_CODE)
  const [consoleOutput, setConsoleOutput] = useState('Hello, World!\n')
  const [steps, setSteps] = useState(0)
  const [wasmReady, setWasmReady] = useState(false)
  const wasmEmuRef = useRef<any | null>(null)

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
  const [flags] = useState({
    zf: 1,
    sf: 0,
    of: 0,
    cf: 0,
    df: 0,
    pf: 1,
  })

  // 1. cd core   /   wasm-pack build --target web --out-dir ../frontend/src/wasm/pkg --dev --out-name web_x86_cor
  // 2. cd frontend   /   npm install   /   npm run dev
  useEffect(() => {
    let mounted = true
    ;(async () => {
      try {
        const wasm = await import('./wasm/pkg/web_x86_core')
        const wasmAny = wasm as any
        const init = wasmAny.default ?? wasmAny.init ?? (() => Promise.resolve())
        await init()
        const Emu = wasm.Emulator
        const emu = new Emu()
        wasmEmuRef.current = emu

        const eip = `0x${emu.get_eip().toString(16).padStart(8, '0')}`
        const eax = `0x${emu.get_eax().toString(16).padStart(8, '0')}`
        const esp = `0x${emu.get_esp().toString(16).padStart(8, '0')}`

        if (mounted) {
          setRegisters((r) => ({ ...r, eip, eax, esp }))
          setConsoleOutput((s) => s + 'WASM: emulator loaded\n')
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

  function onRun() {
    const emu = wasmEmuRef.current
    if (emu) {
      try {
        if (typeof emu.test_add_eax_imm === 'function') {
          emu.test_add_eax_imm()
          setConsoleOutput((s) => s + 'WASM: test_add_eax_imm executed\n')
        } else if (typeof emu.test_push_eax === 'function') {
          emu.test_push_eax()
          setConsoleOutput((s) => s + 'WASM: test_push_eax executed\n')
        } else {
          setConsoleOutput((s) => s + 'WASM: run not implemented\n')
        }
        refreshRegistersFromWasm(emu)
      } catch (e) {
        setConsoleOutput((s) => s + `WASM runtime error: ${String(e)}\n`)
      }
    } else {
      setConsoleOutput((s) => s + 'WASM not ready\n')
    }
  }

  function onStep() {
    const emu = wasmEmuRef.current
    if (emu && wasmReady) {
      try {
        const stepCount = emu.step()
        setSteps(Number(stepCount))
        setConsoleOutput((s) => s + `Step ${stepCount}\n`)
        refreshRegistersFromWasm(emu)
      } catch (e) {
        setConsoleOutput((s) => s + `WASM step error: ${String(e)}\n`)
      }
    } else {
      setConsoleOutput((s) => s + 'WASM not ready\n')
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

  function refreshRegistersFromWasm(emu: any) {
    try {
      const eip = `0x${emu.get_eip().toString(16).padStart(8, '0')}`
      const eax = `0x${emu.get_eax().toString(16).padStart(8, '0')}`
      const esp = `0x${emu.get_esp().toString(16).padStart(8, '0')}`
      setRegisters((r) => ({ ...r, eip, eax, esp }))
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
        </aside>

        <section className="console-pane">
          <div className="console-header">Console</div>
          <pre className="console-output" role="log" aria-live="polite">{consoleOutput}</pre>
        </section>
      </main>
    </div>
  )
}