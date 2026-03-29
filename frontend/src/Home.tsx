import { useNavigate } from 'react-router-dom'
import './App.css'

export default function Home() {
  const navigate = useNavigate()

  return (
    <div className="app-root">
      <header className="topbar">
        <div className="brand">ASU</div>
        <div className="title">Online Assembly x86 Emulator</div>
      </header>

      <main style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: 'calc(100vh - 60px)', gap: '2rem', padding: '2rem' }}>
        <button onClick={() => navigate('/emulator')} className="primary">
          Emulator View
        </button>
        <button onClick={() => navigate('/lab1')} className="primary">
          Lab View
        </button>
      </main>
    </div>
  )
}
