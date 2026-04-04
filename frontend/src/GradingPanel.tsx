import { useState } from 'react'

export interface GradingResult {
  earned: number
  total: number
  autoMax: number
  details: string[]
}

export interface GradingPanelProps {
  labId: number
  description: string
  onSubmit: () => GradingResult | null
  lockedOut?: boolean
}

export default function GradingPanel({
  labId,
  description,
  onSubmit,
  lockedOut = false,
}: GradingPanelProps) {
  const [result, setResult] = useState<GradingResult | null>(null)
  const [submitted, setSubmitted] = useState(false)

  function handleSubmit() {
    if (lockedOut) return
    const r = onSubmit()
    setResult(r)
    setSubmitted(true)
  }

  return (
    <div className="grading-panel">
      <div className="panel-heading" style={{ marginTop: 16 }}>
        Lab {labId} — Autograder
      </div>
      {description && <p className="grading-desc">{description}</p>}

      <button className="submit-btn" onClick={handleSubmit} disabled={lockedOut}>
      {lockedOut ? 'Submit for Grading' : 'Submit for Grading'}
      </button>

      {submitted && result && (
        <div className="grading-result">
          <div
            className="grading-score"
            style={{ color: result.earned >= result.autoMax ? '#2a9d2a' : '#c0392b' }}
          >
            Score: {result.earned} / {result.total}
          </div>
          {result.total > result.autoMax && (
            <p className="grading-manual-note" style={{ color: '#888', fontSize: '0.85em', margin: '4px 0 8px' }}>
              Manual grading ({result.total - result.autoMax} pts) is handled by your instructor.
            </p>
          )}
          <ul className="grading-details">
            {result.details.map((line, i) => (
              <li key={i}>{line}</li>
            ))}
          </ul>
        </div>
      )}

      {submitted && !result && (
        <div className="grading-result">
          <span style={{ color: '#888' }}>
            Run your program first, then submit.
          </span>
        </div>
      )}
    </div>
  )
}
