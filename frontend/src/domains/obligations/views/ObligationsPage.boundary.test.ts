import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('Obligations product boundary', () => {
  it('renders only typed Obligations owner truth and has no legacy REST adapter', () => {
    const store = readFileSync(new URL('../stores/obligations.ts', import.meta.url), 'utf8')
    const view = readFileSync(new URL('./ObligationsWorkspaceView.vue', import.meta.url), 'utf8')

    expect(existsSync(new URL('../api/obligations.ts', import.meta.url))).toBe(false)
    expect(store).toContain('getObligationsCommandClient')
    expect(store).toContain('getObligationsQueryClient')
    expect(store).not.toContain('/api/v1/obligations')
    expect(store).not.toContain('Task')
    expect(store).not.toContain('Decision')
    expect(store).not.toContain('createObligation')
    expect(view).toContain('Manual creation is unavailable')
    expect(view).toContain('surface.setObligationState')
    expect(view).toContain('surface.removeEvidence')
    expect(view).not.toContain('Checklist')
    expect(view).not.toContain('Priority')
  })
})
