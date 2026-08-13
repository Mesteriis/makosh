import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/review/queries/useReviewPageSurface.ts'

export function useReviewSurface() {
  return createDomainSurface({
    surfaceId: 'review',
    labelKey: 'Review',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'review-attention',
        labelKey: 'Attention',
        descriptionKey: 'Review-owned attention requiring an owner decision.',
        icon: 'tabler:clipboard-check',
        status: 'active',
        kind: 'review',
        contract: 'useReviewPageSurface.attention'
      },
      {
        id: 'review-person-match',
        labelKey: 'Person matches',
        descriptionKey: 'Typed person-match candidates owned by Review.',
        icon: 'tabler:users',
        status: 'active',
        kind: 'review',
        contract: 'useReviewPageSurface.personMatchCandidates'
      },
      {
        id: 'review-task-candidates',
        labelKey: 'Task candidates',
        descriptionKey: 'Typed task candidates owned by Review.',
        icon: 'tabler:checkbox',
        status: 'active',
        kind: 'review',
        contract: 'useReviewPageSurface.taskCandidates'
      },
      {
        id: 'review-note-candidates',
        labelKey: 'Note candidates',
        descriptionKey: 'Typed note candidates owned by Review.',
        icon: 'tabler:note',
        status: 'active',
        kind: 'review',
        contract: 'useReviewPageSurface.noteCandidates'
      },
      {
        id: 'review-obligation-candidates',
        labelKey: 'Obligation candidates',
        descriptionKey: 'Evidence-backed obligation candidates owned by Review.',
        icon: 'tabler:contract',
        status: 'active',
        kind: 'review',
        contract: 'useReviewPageSurface.obligationCandidates'
      }
    ],
    childSurfaces: [
      {
        id: 'review-attention',
        labelKey: 'Attention',
        status: 'facade',
        surfacePath,
        capabilityIds: ['review-attention']
      },
      {
        id: 'review-person-match',
        labelKey: 'Person matches',
        status: 'facade',
        surfacePath,
        capabilityIds: ['review-person-match']
      },
      {
        id: 'review-task-candidates',
        labelKey: 'Task candidates',
        status: 'facade',
        surfacePath,
        capabilityIds: ['review-task-candidates']
      },
      {
        id: 'review-note-candidates',
        labelKey: 'Note candidates',
        status: 'facade',
        surfacePath,
        capabilityIds: ['review-note-candidates']
      },
      {
        id: 'review-obligation-candidates',
        labelKey: 'Obligation candidates',
        status: 'facade',
        surfacePath,
        capabilityIds: ['review-obligation-candidates']
      }
    ]
  })
}
