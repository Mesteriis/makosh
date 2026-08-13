import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/knowledge/queries/useKnowledgePageSurface.ts'

export function useKnowledgeSurface() {
  return createDomainSurface({
    surfaceId: 'knowledge',
    labelKey: 'Knowledge',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'knowledge-notes',
        labelKey: 'Knowledge notes',
        descriptionKey: 'Durable owner-local notes with active and archived lifecycle.',
        icon: 'tabler:notebook',
        status: 'active',
        kind: 'query',
        contract: 'useKnowledgePageSurface.notes'
      },
      {
        id: 'knowledge-sources',
        labelKey: 'Public sources',
        descriptionKey: 'Revisioned public source references without provider-private locators.',
        icon: 'tabler:link',
        status: 'active',
        kind: 'command',
        contract: 'useKnowledgePageSurface.sources'
      }
    ],
    childSurfaces: [
      { id: 'knowledge-notes', labelKey: 'Notes', status: 'facade', surfacePath, capabilityIds: ['knowledge-notes'] },
      { id: 'knowledge-sources', labelKey: 'Sources', status: 'facade', surfacePath, capabilityIds: ['knowledge-sources'] }
    ]
  })
}
