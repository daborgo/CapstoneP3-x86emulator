import type { LabConfig } from './labConfig'
import { labConfigs } from './labConfig'

const LAB_CONTENT_STORAGE_KEY = 'labContentOverridesV1'

interface LabConfigOverride {
  title: string
  description: string
  starterCode: string
}

type LabConfigOverrideMap = Record<number, LabConfigOverride>

function readOverrides(): LabConfigOverrideMap {
  try {
    const raw = localStorage.getItem(LAB_CONTENT_STORAGE_KEY)
    if (!raw) return {}

    const parsed = JSON.parse(raw) as LabConfigOverrideMap
    if (!parsed || typeof parsed !== 'object') return {}

    return parsed
  } catch {
    return {}
  }
}

function writeOverrides(overrides: LabConfigOverrideMap) {
  localStorage.setItem(LAB_CONTENT_STORAGE_KEY, JSON.stringify(overrides))
}

function normalizeId(labId: number): number {
  return Number.isInteger(labId) && labId > 0 ? labId : 1
}

export function getLabContent(labId: number): LabConfig {
  const normalizedLabId = normalizeId(labId)
  const fallback = labConfigs[normalizedLabId] ?? labConfigs[1]
  const overrides = readOverrides()
  const override = overrides[normalizedLabId]

  if (!override) return fallback

  return {
    ...fallback,
    title: override.title,
    description: override.description,
    starterCode: override.starterCode,
  }
}

export function saveLabContent(
  labId: number,
  content: Pick<LabConfig, 'title' | 'description' | 'starterCode'>,
): LabConfig {
  const normalizedLabId = normalizeId(labId)
  const fallback = labConfigs[normalizedLabId] ?? labConfigs[1]
  const overrides = readOverrides()

  overrides[normalizedLabId] = {
    title: content.title.trim() || fallback.title,
    description: content.description.trim() || fallback.description,
    starterCode: content.starterCode,
  }

  writeOverrides(overrides)

  return {
    ...fallback,
    ...overrides[normalizedLabId],
  }
}
