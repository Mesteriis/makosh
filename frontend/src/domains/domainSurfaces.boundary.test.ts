import { describe, expect, it } from 'vitest'
import { useAgentsSurface } from './agents/queries/useAgentsSurface'
import { useCalendarSurface } from './calendar/queries/useCalendarSurface'
import { useDocumentsSurface } from './documents/queries/useDocumentsSurface'
import type { DomainSurface } from './domainSurface'
import { useEventTracingSurface } from './event-tracing/queries/useEventTracingSurface'
import { useHomeSurface } from './home/queries/useHomeSurface'
import { useKnowledgeSurface } from './knowledge/queries/useKnowledgeSurface'
import { useOrganizationsSurface } from './organizations/queries/useOrganizationsSurface'
import { usePersonasSurface } from './personas/queries/usePersonasSurface'
import { useProjectsSurface } from './projects/queries/useProjectsSurface'
import { useReviewSurface } from './review/queries/useReviewSurface'
import { useTasksSurface } from './tasks/queries/useTasksSurface'
import { useTimelineSurface } from './timeline/queries/useTimelineSurface'

const plannedDomainSurfaceIds = [
  'agents',
  'calendar',
  'documents',
  'event-tracing',
  'home',
  'knowledge',
  'organizations',
  'personas',
  'projects',
  'review',
  'tasks',
  'timeline'
] as const

function domainSurfaces(): DomainSurface[] {
  return [
    useAgentsSurface(),
    useCalendarSurface(),
    useDocumentsSurface(),
    useEventTracingSurface(),
    useHomeSurface(),
    useKnowledgeSurface(),
    useOrganizationsSurface(),
    usePersonasSurface(),
    useProjectsSurface(),
    useReviewSurface(),
    useTasksSurface(),
    useTimelineSurface()
  ]
}

describe('domain surface scaffolds', () => {
  it('declares planned domain surfaces without Vue-owned business logic', () => {
    const surfaces = domainSurfaces()
    const surfaceIds = surfaces.map((surface) => surface.surfaceId).sort()

    expect(surfaceIds).toEqual([...plannedDomainSurfaceIds].sort())
    expect(new Set(surfaceIds).size).toBe(surfaceIds.length)

    for (const surface of surfaces) {
      expect(surface.ownerLayer).toBe('domain')
      expect(surface.labelKey).toBeTruthy()
      expect(surface.capabilities.length).toBeGreaterThan(0)
      expect(surface.childSurfaces.length).toBeGreaterThan(0)

      const capabilityIds = new Set(surface.capabilities.map((capability) => capability.id))
      for (const child of surface.childSurfaces) {
        expect(child.labelKey).toBeTruthy()
        expect(['active', 'facade', 'planned']).toContain(child.status)

        for (const capabilityId of child.capabilityIds ?? []) {
          expect(capabilityIds.has(capabilityId)).toBe(true)
        }
      }
    }
  })
})
