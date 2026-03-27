import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import './App.css'

export default function Login() {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [role, setRole] = useState('student')
  const [error, setError] = useState('')
  const navigate = useNavigate()

  const handleLogin = (e: React.FormEvent) => {
    e.preventDefault()
    const asuEmail = /^[^@\s]+@asu\.edu$/i
    if (!asuEmail.test(username)) {
      setError('Username must be an @asu.edu email.')
      return
    }
    setError('')
    if (username && password) {
      localStorage.setItem('userRole', role)
      localStorage.setItem('username', username)
      navigate('/lab1')
    }
  }

  const handleSsoLogin = () => {
    localStorage.setItem('userRole', 'student')
    navigate('/auth')
  }

  return (
    <div className="app-root">
      <header className="topbar">
        <div className="brand">ASU</div>
        <div className="title">Online Assembly x86 Emulator</div>
      </header>

      <main style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        minHeight: 'calc(100vh - 60px)',
        padding: '2rem'
      }}>
        <div style={{
          background: 'white',
          padding: '2rem',
          borderRadius: '8px',
          boxShadow: '0 2px 8px rgba(0,0,0,0.1)',
          width: '100%',
          maxWidth: '400px'
        }}>
          <h2 style={{ marginBottom: '1.5rem', textAlign: 'center' }}>Login</h2>

          <button
            type="button"
            className="primary"
            onClick={handleSsoLogin}
            style={{ width: '100%', padding: '0.75rem', fontSize: '1rem', fontWeight: 600, marginBottom: '1rem' }}
          >
            Login with ASU SSO
          </button>

          <div style={{ textAlign: 'center' }}>or</div>

          <form onSubmit={handleLogin} style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
              <label htmlFor="role" style={{ fontWeight: 600 }}>Role</label>
              <select
                id="role"
                value={role}
                onChange={(e) => setRole(e.target.value)}
                style={{
                  padding: '0.75rem',
                  border: '1px solid #ddd',
                  borderRadius: '6px',
                  fontSize: '1rem'
                }}
              >
                <option value="student">Student</option>
                <option value="admin">Instructor/Admin</option>
              </select>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
              <label htmlFor="username" style={{ fontWeight: 600 }}>Username</label>
              <input
                id="username"
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                required
                style={{
                  padding: '0.75rem',
                  border: '1px solid #ddd',
                  borderRadius: '6px',
                  fontSize: '1rem'
                }}
              />
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
              <label htmlFor="password" style={{ fontWeight: 600 }}>Password</label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                style={{
                  padding: '0.75rem',
                  border: '1px solid #ddd',
                  borderRadius: '6px',
                  fontSize: '1rem'
                }}
              />
            </div>

            {error && <div style={{ color: '#ff0000' }}>{error}</div>}
            <button
              type="submit"
              className="primary"
              style={{
                marginTop: '1rem',
                padding: '0.75rem',
                fontSize: '1rem',
                fontWeight: 600
              }}
            >
              Login
            </button>
          </form>
        </div>
      </main>
    </div>
  )
}