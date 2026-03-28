import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import './App.css'
import {
  getLabSubmissions,
  type LabSubmission,
  updateSubmissionFinalScore,
} from './submissionsStore'

export default function InstructorSubmissions() {
  const navigate = useNavigate()
  const [username, setUsername] = useState<string>('Instructor')
  const [submissions, setSubmissions] = useState<LabSubmission[]>([])
  const [scoreDrafts, setScoreDrafts] = useState<Record<string, string>>({})

  function loadSubmissions() {
    const items = getLabSubmissions()
    setSubmissions(items)
    setScoreDrafts((prev) => {
      const next: Record<string, string> = {}
      for (const s of items) {
        next[s.id] = prev[s.id] ?? String(s.finalEarned)
      }
      return next
    })
  }

  useEffect(() => {
    const role = localStorage.getItem('userRole')
    const user = localStorage.getItem('username')

    if (role !== 'admin') {
      navigate('/login')
      return
    }

    setUsername(user || 'Instructor')
    loadSubmissions()
  }, [navigate])

  function handleScoreDraftChange(id: string, value: string) {
    setScoreDrafts((prev) => ({
      ...prev,
      [id]: value,
    }))
  }

  function handleScoreSubmit(id: string, total: number) {
    const raw = (scoreDrafts[id] ?? '').trim()
    const n = Number(raw)
    if (!Number.isFinite(n)) return

    const bounded = Math.max(0, Math.min(n, total))
    updateSubmissionFinalScore(id, bounded)
    setScoreDrafts((prev) => ({
      ...prev,
      [id]: String(bounded),
    }))
    loadSubmissions()
  }

  return (
    <div className="app-root">
      <header className="topbar">
        <div className="brand">ASU</div>
        <div className="title">Instructor Submission Review</div>
        <div style={{ marginLeft: 'auto', paddingRight: '1rem', fontSize: '0.9rem' }}>
          Instructor/Admin: {username}
        </div>
      </header>

      <main style={{ padding: '1.5rem' }}>
        {submissions.length === 0 && (
          <div className="grading-panel">
            <div className="panel-heading">No submissions yet</div>
            <p className="grading-desc">Student lab submissions will appear here after they click Submit for Grading.</p>
          </div>
        )}

        {submissions.length > 0 && (
          <div className="grading-panel">
            <div className="panel-heading" style={{ marginBottom: 12 }}>Submitted Labs</div>
            <div style={{ display: 'grid', gap: '0.75rem' }}>
              {submissions.map((s) => (
                <div
                  key={s.id}
                  style={{
                    border: '1px solid #e2e2e2',
                    borderRadius: 8,
                    padding: '0.9rem',
                    background: '#fff',
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', gap: '1rem', flexWrap: 'wrap' }}>
                    <strong>{s.studentUsername} - Lab {s.labId}</strong>
                    <span style={{ color: '#666', fontSize: '0.9rem' }}>Updated: {new Date(s.updatedAt).toLocaleString()}</span>
                  </div>

                  <div style={{ marginTop: '0.5rem', color: '#444' }}>
                    Auto score: {s.autoEarned} / {s.total}
                  </div>

                  <label style={{ display: 'block', marginTop: '0.5rem', fontWeight: 600 }}>
                    Instructor edited score
                  </label>
                  <input
                    type="text"
                    inputMode="numeric"
                    pattern="[0-9]*"
                    value={scoreDrafts[s.id] ?? String(s.finalEarned)}
                    onChange={(e) => handleScoreDraftChange(s.id, e.target.value)}
                    style={{
                      marginTop: '0.25rem',
                      padding: '0.5rem',
                      border: '1px solid #ccc',
                      borderRadius: 6,
                      width: 140,
                    }}
                  />
                  <button
                    type="button"
                    className="primary"
                    style={{ marginLeft: '0.5rem', padding: '0.5rem 0.75rem' }}
                    onClick={() => handleScoreSubmit(s.id, s.total)}
                  >
                    Submit
                  </button>

                  <details style={{ marginTop: '0.5rem' }}>
                    <summary style={{ cursor: 'pointer' }}>Submission details</summary>
                    <ul style={{ marginTop: '0.5rem' }}>
                      {s.details.map((line, i) => (
                        <li key={i}>{line}</li>
                      ))}
                    </ul>
                  </details>
                </div>
              ))}
            </div>
          </div>
        )}
      </main>
    </div>
  )
}
