import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('Knowledge lifecycle product boundary', () => {
  it('uses generated owner-local Knowledge RPC instead of graph or REST scaffolds', () => {
    const store = readFileSync(new URL('../stores/knowledge.ts', import.meta.url), 'utf8')
    const view = readFileSync(new URL('./KnowledgeWorkspaceView.vue', import.meta.url), 'utf8')

    expect(existsSync(new URL('../api/knowledge.ts', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../types/knowledge.ts', import.meta.url))).toBe(false)
    expect(store).toContain('getKnowledgeCommandClient')
    expect(store).toContain('getKnowledgeQueryClient')
    expect(store).not.toContain('/api/v1/')
    expect(store).not.toContain('graph')
    expect(view).toContain('KnowledgeNoteStateV1')
    expect(view).toContain('Load public sources')
    expect(view).not.toContain('contradiction')
  })
})
