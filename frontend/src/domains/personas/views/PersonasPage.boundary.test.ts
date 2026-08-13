import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('PersonasPage boundary', () => {
  it('uses the generated Persons and Review clients and keeps unavailable projections disabled', () => {
    const viewUrl = new URL('./PersonasWorkspaceView.vue', import.meta.url)
    const workspaceUrl = new URL('../components/PersonasWorkspace.vue', import.meta.url)
    const surfaceUrl = new URL('../queries/usePersonasPageSurface.ts', import.meta.url)
    const apiUrl = new URL('../api/personas.ts', import.meta.url)

    for (const url of [viewUrl, workspaceUrl, surfaceUrl, apiUrl]) {
      expect(existsSync(url)).toBe(true)
    }

    const view = readFileSync(viewUrl, 'utf8')
    const workspace = readFileSync(workspaceUrl, 'utf8')
    const surface = readFileSync(surfaceUrl, 'utf8')
    const api = readFileSync(apiUrl, 'utf8')

    expect(view).toContain('usePersonasPageSurface')
    expect(view).toContain('<PersonasPage :model="model" :actions="actions"')
    expect(view).not.toContain('ApiClient')
    expect(workspace).toContain("activeSection === 'relationships'")
    expect(workspace).toContain('<PersonaRelationshipsPanel')
    expect(surface).toContain('usePersonasQuery')
    expect(surface).toContain('useIdentityCandidatesQuery')
    expect(surface).toContain('useRelationshipsQuery')
    expect(surface).not.toContain('useIdentityTracesQuery')
    expect(api).toContain('PersonsQueryService')
    expect(api).toContain('ReviewPersonMatchCandidateQueryService')
    expect(api).toContain('getRelationshipsQueryClient')
    expect(api).not.toContain('relationships_unavailable')
    expect(api).not.toContain('ApiClient')
    expect(api).not.toMatch(/\/api\/v1\//)
  })

  it('keeps legacy Persons frontend paths retired', () => {
    const retiredPaths = [
      '../../../app/queries/usePersonsViewSurface.ts',
      '../queries/usePersonsPageSurface.ts',
      '../queries/usePersonsSurface.ts',
      './PersonsPage.boundary.test.ts',
      '../../../../stories/app/Persons.stories.ts'
    ]

    for (const retiredPath of retiredPaths) {
      expect(existsSync(new URL(retiredPath, import.meta.url)), retiredPath).toBe(false)
    }
  })
})
