import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('Projects product boundary', () => {
  it('renders only typed owner Projects lifecycle, outcomes and references', () => {
    const view = readFileSync(new URL('./ProjectsWorkspaceView.vue', import.meta.url), 'utf8')
    const surface = readFileSync(new URL('../queries/useProjectsPageSurface.ts', import.meta.url), 'utf8')
    const store = readFileSync(new URL('../stores/projects.ts', import.meta.url), 'utf8')
    const api = readFileSync(new URL('../api/projects.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./ProjectsPage.vue', import.meta.url))).toBe(false)
    expect(view).toContain('OWNER PROJECTS')
    expect(view).toContain('Expected outcomes')
    expect(view).toContain('Typed public references')
    expect(view).not.toContain('Timeline')
    expect(view).not.toContain('Messages')
    expect(view).not.toContain('Graph')
    expect(surface).toContain('useProjectsStore')
    expect(surface).toContain('selectedProject')
    expect(surface).toContain('outcomes')
    expect(surface).toContain('references')
    expect(store).toContain('getProjectsQueryClient')
    expect(store).toContain('getProjectsCommandClient')
    expect(api).not.toContain('/api/v1/projects')
  })
})
