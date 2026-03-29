export interface LabSubmission {
  id: string
  labId: number
  studentUsername: string
  autoEarned: number
  total: number
  finalEarned: number
  details: string[]
  submittedAt: string
  updatedAt: string
}

const STORAGE_KEY = 'labSubmissions'

function readRaw(): LabSubmission[] {
  const raw = localStorage.getItem(STORAGE_KEY)
  if (!raw) return []

  try {
    const parsed = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    return parsed as LabSubmission[]
  } catch {
    return []
  }
}

function writeRaw(items: LabSubmission[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(items))
}

export function getLabSubmissions(): LabSubmission[] {
  return readRaw().sort((a, b) => {
    return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
  })
}

export function saveStudentSubmission(input: {
  labId: number
  studentUsername: string
  autoEarned: number
  total: number
  details: string[]
}) {
  const items = readRaw()
  const now = new Date().toISOString()
  const idx = items.findIndex(
    (x) => x.labId === input.labId && x.studentUsername.toLowerCase() === input.studentUsername.toLowerCase(),
  )

  if (idx >= 0) {
    items[idx] = {
      ...items[idx],
      autoEarned: input.autoEarned,
      total: input.total,
      finalEarned: input.autoEarned,
      details: input.details,
      updatedAt: now,
    }
  } else {
    items.push({
      id: `${input.studentUsername.toLowerCase()}-lab${input.labId}`,
      labId: input.labId,
      studentUsername: input.studentUsername,
      autoEarned: input.autoEarned,
      total: input.total,
      finalEarned: input.autoEarned,
      details: input.details,
      submittedAt: now,
      updatedAt: now,
    })
  }

  writeRaw(items)
}

export function updateSubmissionFinalScore(id: string, finalEarned: number) {
  const items = readRaw()
  const idx = items.findIndex((x) => x.id === id)
  if (idx < 0) return

  const bounded = Math.max(0, Math.min(finalEarned, items[idx].total))
  items[idx] = {
    ...items[idx],
    finalEarned: bounded,
    updatedAt: new Date().toISOString(),
  }

  writeRaw(items)
}
