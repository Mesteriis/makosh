import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/obligations/queries/useObligationsPageSurface.ts'

export function useObligationsSurface() {
  return createDomainSurface({
    surfaceId: 'obligations',
    labelKey: 'Obligations',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'obligations-worklist',
        labelKey: 'Obligation worklist',
        descriptionKey: 'Durable owner obligations, state transitions and due context.',
        icon: 'tabler:checkbox',
        status: 'active',
        kind: 'query',
        contract: 'useObligationsPageSurface.obligations'
      },
      {
        id: 'obligations-evidence',
        labelKey: 'Obligation evidence',
        descriptionKey: 'Typed public evidence links and confirmed party identities.',
        icon: 'tabler:link',
        status: 'active',
        kind: 'command',
        contract: 'useObligationsPageSurface.evidence'
      }
    ],
    childSurfaces: [
      {
        id: 'obligations-list',
        labelKey: 'Obligations',
        status: 'facade',
        surfacePath,
        capabilityIds: ['obligations-worklist']
      },
      {
        id: 'obligations-evidence',
        labelKey: 'Evidence',
        status: 'facade',
        surfacePath,
        capabilityIds: ['obligations-evidence']
      }
    ]
  })
}
