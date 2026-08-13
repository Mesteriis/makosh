import { describe, expect, it } from 'vitest'
import { relationshipGraphEdgeData } from './personaRelationshipGraphModel'

describe('persona relationship graph model', () => {
  it('accepts complete edge data', () => {
    expect(relationshipGraphEdgeData({
      relationshipId: 'relationship-1',
      type: 'colleague',
      state: 'confirmed',
      revision: 2,
      validFrom: '2026-01-01T00:00:00.000Z',
      validUntil: null,
      sourceTitle: 'Alice',
      targetTitle: 'Acme',
      icon: 'tabler:briefcase',
      iconLabel: 'Colleague',
    })).toEqual({
      relationshipId: 'relationship-1',
      type: 'colleague',
      state: 'confirmed',
      revision: 2,
      validFrom: '2026-01-01T00:00:00.000Z',
      validUntil: null,
      sourceTitle: 'Alice',
      targetTitle: 'Acme',
      icon: 'tabler:briefcase',
      iconLabel: 'Colleague',
    })
  })

  it('rejects malformed edge data', () => {
    expect(relationshipGraphEdgeData({ relationshipId: 'relationship-1' })).toBeNull()
    expect(relationshipGraphEdgeData(null)).toBeNull()
  })
})
