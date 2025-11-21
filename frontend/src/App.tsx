import { useEffect, useState } from 'react'
import init, { Emulator } from './wasm/pkg/web_x86_core'

export default function App() {
  const [emu, setEmu] = useState<Emulator | null>(null)
  const [steps, setSteps] = useState(0)

  useEffect(() => {
    (async () => {
      await init()
      setEmu(new Emulator())
    })()
  }, [])

  return (
    <div style={{ padding: 20 }}>
      <h1>Web x86 Emulator</h1>
      <button disabled={!emu} onClick={() => setSteps(emu?.step() || 0)}>Step</button>
      <div>Steps: {steps}</div>
    </div>
  )
}
