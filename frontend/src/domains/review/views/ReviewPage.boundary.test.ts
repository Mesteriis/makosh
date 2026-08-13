import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ReviewPage boundary', () => {
  it('exposes only the five typed Review-owned slices', () => {
    const surfaceSource = readFileSync(
      new URL('../queries/useReviewPageSurface.ts', import.meta.url),
      'utf8'
    )
    const storeSource = readFileSync(new URL('../stores/review.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./ReviewPage.vue', import.meta.url))).toBe(false)
    expect(surfaceSource).toContain('attention = computed')
    expect(surfaceSource).toContain('personMatchCandidates = computed')
    expect(surfaceSource).toContain('taskCandidates = computed')
    expect(surfaceSource).toContain('noteCandidates = computed')
    expect(surfaceSource).toContain('obligationCandidates = computed')
    expect(storeSource).toContain('getReviewAttentionQueryClient')
    expect(storeSource).toContain('getReviewPersonMatchCandidateQueryClient')
    expect(storeSource).toContain('getReviewTaskCandidateQueryClient')
    expect(storeSource).toContain('getReviewNoteCandidateQueryClient')
    expect(storeSource).toContain('getReviewObligationCandidateQueryClient')
    expect(`${surfaceSource}\n${storeSource}`).not.toMatch(
      /Relationship|Contradiction|target_domain|metadata|\/api\/v1\/review/
    )
  })
})
